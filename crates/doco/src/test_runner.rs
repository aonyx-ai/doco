//! Test runner for Doco's end-to-end tests

use anyhow::Context;
use testcontainers::core::{Host, IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

use crate::{Client, Doco, Result};

/// The host name for Docker containers to access the host machine
///
/// Docker containers can access the host using this internal hostname. The hostname is
/// automatically set on macOS and Windows hosts, but on Linux hosts it must be set explicitly by
/// Doco.
const DOCKER_HOST: &str = "host.docker.internal";

/// Test runner for Doco's end-to-end tests
///
/// The `TestRunner` is responsible for executing each test in an isolated, ephemeral environment.
/// It starts Selenium in a container, configures the WebDriver [`Client`] to connect to Selenium,
/// and then runs each test against a clean instance of the server and its services.
///
/// It should not be necessary to use this struct directly. Instead, use the [`doco::main`] and
/// [`doco::test`] macros to automatically set up the test runner, collect all tests, and pass them
/// to the runner.
#[derive(Debug)]
pub struct TestRunner {
    /// The Doco configuration to use for the tests
    doco: Doco,

    /// The running Selenium container to which the WebDriver client connects
    selenium: ContainerAsync<GenericImage>,
}

impl TestRunner {
    /// Initialize the test runner with the given Doco configuration
    ///
    /// This method starts the Selenium container and returns a new `TestRunner` instance. Since
    /// starting the container can fail, this method returns a `Result` that must be handled.
    pub async fn init(doco: Doco) -> Result<Self> {
        println!("Initializing ephemeral test environment...");

        let selenium = start_selenium().await?;

        Ok(Self { doco, selenium })
    }

    /// Run the given test in the ephemeral environment
    ///
    /// This method executes a test in a clean, ephemeral environment. First, it starts any
    /// auxiliary services like databases and waits for them to be ready. Then, it starts the
    /// server, configures the WebDriver [`Client`], and calls the test function.
    pub async fn run(&self, test: fn(Client) -> Result<()>) -> Result<()> {
        let mut services = Vec::with_capacity(self.doco.services().len());

        let mut server = GenericImage::new(self.doco.server().image(), self.doco.server().tag())
            .with_exposed_port(self.doco.server().port().tcp());

        if let Some(wait) = self.doco.server.wait() {
            server = server.with_wait_for(wait.clone());
        }

        let mut server = server.with_host(DOCKER_HOST, Host::HostGateway);

        for service in self.doco.services() {
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

        let server = server.start().await?;
        let port = server.get_host_port_ipv4(self.doco.server().port()).await?;

        let driver = thirtyfour::WebDriver::new(
            &format!(
                "http://{}:{}",
                self.selenium.get_host().await?,
                self.selenium.get_host_port_ipv4(4444).await?
            ),
            thirtyfour::DesiredCapabilities::firefox(),
        )
        .await
        .expect("failed to connect to WebDriver");

        let client = Client::builder()
            .base_url(format!("http://{DOCKER_HOST}:{port}").parse()?)
            .client(driver.clone())
            .build();

        test(client)?;

        driver.quit().await.ok();

        Ok(())
    }
}

/// Start the Selenium container
///
/// This function starts the Selenium container, waits for it to be ready, and then returns a
/// reference to the running container. For compatibility between macOS, Linux, and Windows, the
/// [`DOCKER_HOST`] is set explicitly on all platforms.
async fn start_selenium() -> Result<ContainerAsync<GenericImage>> {
    GenericImage::new("selenium/standalone-firefox", "latest")
        .with_exposed_port(4444.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Started Selenium Standalone"))
        .with_host(DOCKER_HOST, Host::HostGateway)
        .start()
        .await
        .context("failed to start Selenium container")
}

#[cfg(test)]
mod tests {
    use axum::routing::get;
    use axum::Router;
    use tokio::net::TcpListener;

    use crate::test_utils::*;
    use crate::Result;

    use super::*;

    #[tokio::test]
    async fn selenium_can_access_host() -> Result<()> {
        let listener = TcpListener::bind("0.0.0.0:0").await?;
        let port = listener.local_addr()?.port();

        let app = Router::new().route("/", get(|| async { "hello from the test" }));
        tokio::spawn(async { axum::serve(listener, app).await });

        let selenium = start_selenium().await?;

        let driver = thirtyfour::WebDriver::new(
            &format!(
                "http://{}:{}",
                selenium.get_host().await?,
                selenium.get_host_port_ipv4(4444).await?
            ),
            thirtyfour::DesiredCapabilities::firefox(),
        )
        .await
        .expect("failed to connect to WebDriver");

        driver
            .goto(&format!("http://{DOCKER_HOST}:{port}/"))
            .await?;
        let body = driver.source().await?;

        assert!(body.contains("hello from the test"));

        driver.quit().await.ok();

        Ok(())
    }

    #[test]
    fn trait_send() {
        assert_send::<TestRunner>();
    }

    #[test]
    fn trait_sync() {
        assert_sync::<TestRunner>();
    }

    #[test]
    fn trait_unpin() {
        assert_unpin::<TestRunner>();
    }
}
