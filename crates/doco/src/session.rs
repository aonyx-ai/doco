//! Long-lived browser session for reusing a single connection across multiple operations

use std::ops::Deref;
use std::sync::Arc;

use getset::Getters;
use reqwest::Url;
use testcontainers::core::{Host, IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tracing::{debug, info};

use crate::{Client, ConnectError, Doco, Headless};

/// The host name for Docker containers to access the host machine
const DOCKER_HOST: &str = "host.docker.internal";

/// A running service container with its resolved network address
///
/// Created by starting a [`Service`] configuration. The container is kept alive by ownership and
/// stopped when dropped.
///
/// [`Service`]: crate::Service
struct RunningService {
    /// The running container
    container: ContainerAsync<GenericImage>,

    /// The service image name, used to register a DNS host entry on the server
    image: String,
}

impl RunningService {
    /// Starts a service container from its configuration
    async fn start(config: &crate::Service) -> Result<Self, ConnectError> {
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

        let container = image
            .start()
            .await
            .map_err(|source| ConnectError::StartService {
                image: config.image().clone(),
                source,
            })?;

        debug!(image = config.image(), "service container ready");

        Ok(Self {
            container,
            image: config.image().clone(),
        })
    }

    /// Returns the bridge IP address for linking this service to the server container
    async fn bridge_ip(&self) -> Result<std::net::IpAddr, ConnectError> {
        let ip = self
            .container
            .get_bridge_ip_address()
            .await
            .map_err(|source| ConnectError::ResolveServiceAddress {
                image: self.image.clone(),
                source,
            })?;
        debug!(image = self.image, %ip, "resolved service bridge IP");
        Ok(ip)
    }
}

/// A running server container with its resolved base URL
///
/// Created by starting a [`Server`] configuration along with any [`RunningService`]s that it
/// depends on. The container is kept alive by ownership.
///
/// [`Server`]: crate::Server
struct RunningServer {
    /// The running container
    container: ContainerAsync<GenericImage>,

    /// The base URL accessible from the host (e.g. `http://host.docker.internal:32789`)
    base_url: Url,
}

#[bon::bon]
impl RunningServer {
    /// Starts the server container, linking it to any running services
    #[builder]
    async fn start(
        config: &crate::Server,
        services: &[RunningService],
    ) -> Result<Self, ConnectError> {
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

        let container = server
            .start()
            .await
            .map_err(|source| ConnectError::StartServer { source })?;
        let port = container
            .get_host_port_ipv4(config.port())
            .await
            .map_err(|source| ConnectError::ResolveServerPort { source })?;
        let base_url = Url::parse(&format!("http://{DOCKER_HOST}:{port}"))
            .map_err(|source| ConnectError::ParseServerUrl { source })?;

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
) -> Result<thirtyfour::WebDriver, ConnectError> {
    let mut caps = thirtyfour::DesiredCapabilities::firefox();
    match doco.headless() {
        Headless::Enabled => caps
            .set_headless()
            .map_err(|source| ConnectError::SetHeadless { source })?,
        Headless::Disabled => {}
    }
    if !doco.prefs().is_empty() {
        let mut prefs = thirtyfour::common::capabilities::firefox::FirefoxPreferences::new();
        for (name, value) in doco.prefs() {
            prefs
                .set(name, value)
                .map_err(|source| ConnectError::SetPreference {
                    name: name.clone(),
                    source,
                })?;
        }
        caps.set_preferences(prefs)
            .map_err(|source| ConnectError::ApplyPreferences { source })?;
    }

    let host = selenium
        .get_host()
        .await
        .map_err(|source| ConnectError::ResolveSeleniumEndpoint { source })?;
    let port = selenium
        .get_host_port_ipv4(4444)
        .await
        .map_err(|source| ConnectError::ResolveSeleniumEndpoint { source })?;
    let endpoint = format!("http://{host}:{port}");

    debug!(
        headless = ?doco.headless(),
        %endpoint,
        "connecting to WebDriver",
    );

    let driver = thirtyfour::WebDriver::new(&endpoint, caps)
        .await
        .map_err(|source| ConnectError::ConnectWebDriver { source })?;

    if let Some(viewport) = doco.viewport() {
        debug!(
            width = viewport.width(),
            height = viewport.height(),
            "setting browser viewport",
        );
        driver
            .set_window_rect(0, 0, viewport.width(), viewport.height())
            .await
            .map_err(|source| ConnectError::SetViewport { source })?;
    }

    Ok(driver)
}

/// A long-lived browser session connected to a running application
///
/// A `Session` holds a [`Client`] and the container handles that keep Selenium, the application
/// server, and any services alive. When the session is dropped, the containers are stopped
/// automatically.
///
/// Use [`Doco::connect()`] to create a session.
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
/// session.close().await;
/// # Ok(())
/// # }
/// ```
///
/// [`Doco::connect()`]: crate::Doco::connect
#[derive(Getters)]
pub struct Session {
    /// The WebDriver client connected to the application
    #[getset(get = "pub")]
    client: Client,

    /// The WebDriver instance, kept for cleanup
    driver: thirtyfour::WebDriver,

    /// The Selenium container, kept alive by ownership
    _selenium: Arc<ContainerAsync<GenericImage>>,

    /// The application server container, kept alive by ownership
    _server: ContainerAsync<GenericImage>,

    /// Auxiliary service containers, kept alive by ownership
    _services: Vec<ContainerAsync<GenericImage>>,
}

#[bon::bon]
impl Session {
    /// Creates a new session by starting all required containers
    ///
    /// Starts Selenium, the application server, and any configured services, then connects a
    /// WebDriver client. This is the implementation behind [`Doco::connect()`].
    pub(crate) async fn connect(doco: &Doco) -> Result<Self, ConnectError> {
        info!("initializing session");
        let selenium = Arc::new(Self::start_selenium().await?);
        Self::with_selenium()
            .doco(doco)
            .selenium(selenium)
            .call()
            .await
    }

    /// Creates a session using an existing Selenium container
    ///
    /// Used by [`TestRunner`] to share one Selenium instance across tests while creating fresh
    /// server and service containers per test.
    ///
    /// [`TestRunner`]: crate::TestRunner
    #[builder]
    pub(crate) async fn with_selenium(
        doco: &Doco,
        selenium: Arc<ContainerAsync<GenericImage>>,
    ) -> Result<Self, ConnectError> {
        let mut services = Vec::with_capacity(doco.services().len());
        for config in doco.services() {
            services.push(RunningService::start(config).await?);
        }

        let server = RunningServer::start()
            .config(doco.server())
            .services(&services)
            .call()
            .await?;
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
    /// the session is dropped, but calling this method ensures the browser exits cleanly. The
    /// method ignores a WebDriver quit error, because the containers are cleaned up on drop
    /// regardless.
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
    /// session.close().await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(self) {
        debug!("closing session");
        self.driver.quit().await.ok();
    }

    /// Starts the Selenium container
    ///
    /// Exposed for [`TestRunner`] to share a single Selenium instance across multiple tests. Most
    /// callers should use [`Doco::connect()`] instead.
    ///
    /// [`TestRunner`]: crate::TestRunner
    pub(crate) async fn start_selenium() -> Result<ContainerAsync<GenericImage>, ConnectError> {
        info!("starting selenium container");
        GenericImage::new("selenium/standalone-firefox", "latest")
            .with_exposed_port(4444.tcp())
            .with_wait_for(WaitFor::message_on_stdout("Started Selenium Standalone"))
            .with_host(DOCKER_HOST, Host::HostGateway)
            .start()
            .await
            .map_err(|source| ConnectError::StartSelenium { source })
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
    use super::*;

    fn assert_auto_traits<T: Send + Sync + Unpin>() {}

    #[test]
    fn running_server_is_send_sync_and_unpin() {
        assert_auto_traits::<RunningServer>();
    }

    #[test]
    fn running_service_is_send_sync_and_unpin() {
        assert_auto_traits::<RunningService>();
    }

    #[test]
    fn session_is_send_sync_and_unpin() {
        assert_auto_traits::<Session>();
    }
}
