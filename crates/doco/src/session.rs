//! Long-lived browser session for reusing a single connection across multiple operations

use std::ops::Deref;
use std::sync::Arc;

use anyhow::Context;
use getset::Getters;
use reqwest::Url;
use testcontainers::core::{Host, IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tracing::{debug, info};

use crate::{Client, Doco, Result};

/// The host name for Docker containers to access the host machine
const DOCKER_HOST: &str = "host.docker.internal";

/// A running service container with its resolved network address
///
/// Created by starting a [`Service`](crate::Service) configuration. The container is kept alive
/// by ownership and stopped when dropped.
struct RunningService {
    /// The running container
    container: ContainerAsync<GenericImage>,

    /// The service image name, used to register a DNS host entry on the server
    image: String,
}

impl RunningService {
    /// Starts a service container from its configuration
    async fn start(config: &crate::Service) -> Result<Self> {
        debug!(
            image = config.image(),
            tag = config.tag(),
            "starting service container"
        );

        let mut image = GenericImage::new(config.image(), config.tag());

        if let Some(wait) = config.wait() {
            image = image.with_wait_for(wait.clone());
        }

        let mut image = image.with_host("doco", Host::HostGateway);

        for env in config.envs() {
            image = image.with_env_var(env.name().clone(), env.value().clone());
        }

        for mount in config.mounts() {
            image = image.with_mount(mount.clone());
        }

        if !config.cmd().is_empty() {
            image = image.with_cmd(config.cmd().clone());
        }

        let container = image.start().await?;

        debug!(image = config.image(), "service container ready");

        Ok(Self {
            container,
            image: config.image().clone(),
        })
    }

    /// The bridge IP address for linking this service to the server container
    async fn bridge_ip(&self) -> Result<std::net::IpAddr> {
        let ip = self
            .container
            .get_bridge_ip_address()
            .await
            .context("failed to get bridge IP for service")?;
        debug!(image = self.image, %ip, "resolved service bridge IP");
        Ok(ip)
    }
}

/// A running server container with its resolved base URL
///
/// Created by starting a [`Server`](crate::Server) configuration along with any
/// [`RunningService`]s that it depends on. The container is kept alive by ownership.
struct RunningServer {
    /// The running container
    container: ContainerAsync<GenericImage>,

    /// The base URL accessible from the host (e.g. `http://host.docker.internal:32789`)
    base_url: Url,
}

impl RunningServer {
    /// Starts the server container, linking it to any running services
    async fn start(config: &crate::Server, services: &[RunningService]) -> Result<Self> {
        debug!(
            image = config.image(),
            tag = config.tag(),
            port = config.port(),
            "starting server container",
        );

        let mut server =
            GenericImage::new(config.image(), config.tag()).with_exposed_port(config.port().tcp());

        if let Some(wait) = config.wait() {
            server = server.with_wait_for(wait.clone());
        }

        let mut server = server.with_host(DOCKER_HOST, Host::HostGateway);

        for service in services {
            let ip = service.bridge_ip().await?;
            debug!(service = service.image, %ip, "linking service to server");
            server = server.with_host(&service.image, Host::Addr(ip));
        }

        for env in config.envs() {
            server = server.with_env_var(env.name().clone(), env.value().clone());
        }

        for mount in config.mounts() {
            server = server.with_mount(mount.clone());
        }

        if !config.cmd().is_empty() {
            server = server.with_cmd(config.cmd().clone());
        }

        let container = server.start().await?;
        let port = container.get_host_port_ipv4(config.port()).await?;
        let base_url = format!("http://{DOCKER_HOST}:{port}").parse()?;

        debug!(%base_url, "server container ready");

        Ok(Self {
            container,
            base_url,
        })
    }
}

/// Creates a WebDriver instance connected to a Selenium container
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

    let endpoint = format!(
        "http://{}:{}",
        selenium.get_host().await?,
        selenium.get_host_port_ipv4(4444).await?
    );

    debug!(
        headless = *doco.headless(),
        %endpoint,
        "connecting to WebDriver",
    );

    let driver = thirtyfour::WebDriver::new(&endpoint, caps)
        .await
        .context("failed to connect to WebDriver")?;

    if let Some(viewport) = doco.viewport() {
        debug!(
            width = viewport.width(),
            height = viewport.height(),
            "setting browser viewport",
        );
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
/// # Examples
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
    /// Creates a new session by starting all required containers
    ///
    /// Starts Selenium, the application server, and any configured services, then connects a
    /// WebDriver client. This is the implementation behind [`Doco::connect()`].
    pub(crate) async fn connect(doco: &Doco) -> Result<Self> {
        info!("initializing session");
        let selenium = Arc::new(Self::start_selenium().await?);
        Self::with_selenium(doco, selenium).await
    }

    /// Creates a session using an existing Selenium container
    ///
    /// Used by [`TestRunner`](crate::TestRunner) to share one Selenium instance across tests
    /// while creating fresh server and service containers per test.
    pub(crate) async fn with_selenium(
        doco: &Doco,
        selenium: Arc<ContainerAsync<GenericImage>>,
    ) -> Result<Self> {
        let mut services = Vec::with_capacity(doco.services().len());
        for config in doco.services() {
            services.push(RunningService::start(config).await?);
        }

        let server = RunningServer::start(doco.server(), &services).await?;
        let driver = create_driver(&selenium, doco).await?;

        let client = Client::builder()
            .base_url(server.base_url)
            .client(driver.clone())
            .build();

        Ok(Self {
            client,
            driver,
            _selenium: selenium,
            _server: server.container,
            _services: services.into_iter().map(|s| s.container).collect(),
        })
    }

    /// Shuts down the browser session
    ///
    /// Sends a quit command to the WebDriver. The containers are cleaned up automatically when
    /// the session is dropped, but calling this method ensures the browser exits cleanly.
    ///
    /// # Errors
    ///
    /// This method currently always succeeds. The WebDriver quit error is intentionally
    /// suppressed since the containers will be cleaned up on drop regardless.
    pub async fn close(self) -> Result<()> {
        debug!("closing session");
        self.driver.quit().await.ok();
        Ok(())
    }

    /// Starts the Selenium container
    ///
    /// Exposed for [`TestRunner`](crate::TestRunner) to share a single Selenium instance across
    /// multiple tests. Most callers should use [`Doco::connect()`] instead.
    pub(crate) async fn start_selenium() -> Result<ContainerAsync<GenericImage>> {
        info!("starting selenium container");
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
