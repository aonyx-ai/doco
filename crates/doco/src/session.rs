//! Long-lived browser session for reusing a single connection across multiple operations

use std::ops::Deref;
use std::sync::Arc;

use anyhow::Context;
use getset::Getters;
use reqwest::Url;
use testcontainers::core::{Host, IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

use crate::{Client, Doco, Result};

/// The host name for Docker containers to access the host machine
const DOCKER_HOST: &str = "host.docker.internal";

/// Running application containers with a resolved base URL
///
/// Holds the server and service containers alive and exposes the base URL that the browser should
/// target. Created by [`Environment::start()`] and consumed by [`Session`] construction.
struct Environment {
    /// The base URL of the running application (e.g. `http://host.docker.internal:32789`)
    base_url: Url,

    /// The application server container
    server: ContainerAsync<GenericImage>,

    /// Auxiliary service containers (databases, caches, etc.)
    services: Vec<ContainerAsync<GenericImage>>,
}

impl Environment {
    /// Start the application server and any configured services
    ///
    /// Launches service containers first (so their addresses can be linked into the server), then
    /// starts the server container and resolves its host-accessible port into a base URL.
    async fn start(doco: &Doco) -> Result<Self> {
        let mut services = Vec::with_capacity(doco.services().len());

        let mut server = GenericImage::new(doco.server().image(), doco.server().tag())
            .with_exposed_port(doco.server().port().tcp());

        if let Some(wait) = doco.server().wait() {
            server = server.with_wait_for(wait.clone());
        }

        let mut server = server.with_host(DOCKER_HOST, Host::HostGateway);

        for service in doco.services() {
            let mut image = GenericImage::new(service.image(), service.tag());

            if let Some(wait) = service.wait() {
                image = image.with_wait_for(wait.clone());
            }

            let mut image = image.with_host("doco", Host::HostGateway);

            for env in service.envs() {
                image = image.with_env_var(env.name().clone(), env.value().clone());
            }

            let container = image.start().await?;

            server = server.with_host(
                service.image(),
                Host::Addr(container.get_bridge_ip_address().await?),
            );

            services.push(container);
        }

        for env in doco.server().envs() {
            server = server.with_env_var(env.name().clone(), env.value().clone());
        }

        let server = server.start().await?;
        let port = server.get_host_port_ipv4(doco.server().port()).await?;
        let base_url = format!("http://{DOCKER_HOST}:{port}").parse()?;

        Ok(Self {
            base_url,
            server,
            services,
        })
    }
}

/// Create a WebDriver instance connected to a Selenium container
///
/// Configures Firefox capabilities (headless mode) and viewport dimensions based on the [`Doco`]
/// configuration, then connects to the Selenium WebDriver endpoint.
async fn create_driver(
    selenium: &ContainerAsync<GenericImage>,
    doco: &Doco,
) -> Result<thirtyfour::WebDriver> {
    let mut caps = thirtyfour::DesiredCapabilities::firefox();
    if *doco.headless() {
        caps.set_headless()
            .context("failed to set headless capability")?;
    }

    let driver = thirtyfour::WebDriver::new(
        &format!(
            "http://{}:{}",
            selenium.get_host().await?,
            selenium.get_host_port_ipv4(4444).await?
        ),
        caps,
    )
    .await
    .context("failed to connect to WebDriver")?;

    if let Some(viewport) = doco.viewport() {
        driver
            .set_window_rect(0, 0, viewport.width(), viewport.height())
            .await
            .context("failed to set browser viewport")?;
    }

    Ok(driver)
}

/// A long-lived browser session connected to a running application
///
/// A `Session` holds a [`Client`] and the container handles that keep Selenium, the application
/// server, and any services alive. When the session is dropped, the containers are stopped
/// automatically.
///
/// Use [`Doco::connect()`](crate::Doco::connect) to create a session.
///
/// # Example
///
/// ```no_run
/// use doco::{Doco, Server};
///
/// # async fn example() -> doco::Result<()> {
/// let doco = Doco::builder()
///     .server(Server::builder().image("my-app").tag("latest").port(8080).build())
///     .build();
///
/// let session = doco.connect().await?;
///
/// session.goto("/").await?;
/// let body = session.source().await?;
/// assert!(body.contains("Hello"));
///
/// session.close().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Getters)]
pub struct Session {
    /// The WebDriver client connected to the application
    #[getset(get = "pub")]
    client: Client,

    /// The WebDriver instance, kept for cleanup
    driver: thirtyfour::WebDriver,

    /// The Selenium container — kept alive by ownership
    _selenium: Arc<ContainerAsync<GenericImage>>,

    /// The application server container — kept alive by ownership
    _server: ContainerAsync<GenericImage>,

    /// Auxiliary service containers — kept alive by ownership
    _services: Vec<ContainerAsync<GenericImage>>,
}

impl Session {
    /// Create a new session by starting all required containers
    ///
    /// Starts Selenium, the application server, and any configured services, then connects a
    /// WebDriver client. This is the implementation behind [`Doco::connect()`].
    pub(crate) async fn connect(doco: &Doco) -> Result<Self> {
        println!("Initializing session...");
        let selenium = Arc::new(Self::start_selenium().await?);
        Self::with_selenium(doco, selenium).await
    }

    /// Create a session using an existing Selenium container
    ///
    /// Used by [`TestRunner`](crate::TestRunner) to share one Selenium instance across tests
    /// while creating fresh server and service containers per test.
    pub(crate) async fn with_selenium(
        doco: &Doco,
        selenium: Arc<ContainerAsync<GenericImage>>,
    ) -> Result<Self> {
        let env = Environment::start(doco).await?;
        let driver = create_driver(&selenium, doco).await?;

        let client = Client::builder()
            .base_url(env.base_url)
            .client(driver.clone())
            .build();

        Ok(Self {
            client,
            driver,
            _selenium: selenium,
            _server: env.server,
            _services: env.services,
        })
    }

    /// Shut down the browser session
    ///
    /// This sends a quit command to the WebDriver. The containers are cleaned up automatically
    /// when the session is dropped, but calling this method ensures the browser exits cleanly.
    pub async fn close(self) -> Result<()> {
        self.driver.quit().await.ok();
        Ok(())
    }

    /// Start the Selenium container
    ///
    /// This is exposed for [`TestRunner`](crate::TestRunner) to share a single Selenium instance
    /// across multiple tests. Most callers should use [`Doco::connect()`] instead.
    pub(crate) async fn start_selenium() -> Result<ContainerAsync<GenericImage>> {
        GenericImage::new("selenium/standalone-firefox", "latest")
            .with_exposed_port(4444.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Started Selenium Standalone"))
            .with_host(DOCKER_HOST, Host::HostGateway)
            .start()
            .await
            .context("failed to start Selenium container")
    }
}

impl Deref for Session {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;

    use super::*;

    #[test]
    fn trait_send() {
        assert_send::<Session>();
    }

    #[test]
    fn trait_sync() {
        assert_sync::<Session>();
    }

    #[test]
    fn trait_unpin() {
        assert_unpin::<Session>();
    }
}
