//! Test runner for Doco's end-to-end tests

use std::future::Future;
use std::sync::Arc;

use tracing::{info, info_span};

use crate::{Client, Doco, Result, Session, TestCase};

/// Test runner for Doco's end-to-end tests
///
/// The `TestRunner` is responsible for executing each test in an isolated, ephemeral environment.
/// It starts Selenium in a container, configures the WebDriver [`Client`] to connect to Selenium,
/// and then runs each test against a clean instance of the server and its services.
///
/// It should not be necessary to use this struct directly. Instead, use the [`doco::main`] and
/// [`doco::test`] macros to automatically set up the test runner, collect all tests, and pass them
/// to the runner.
pub struct TestRunner {
    /// The tokio runtime used to run async test code
    rt: tokio::runtime::Runtime,

    /// The Doco configuration to use for the tests
    doco: Doco,

    /// The running Selenium container, shared across tests via [`Session`]
    selenium: Arc<testcontainers::ContainerAsync<testcontainers::GenericImage>>,
}

impl TestRunner {
    /// Builds the tokio runtime, runs the user's async init block, and initializes the runner
    ///
    /// # Panics
    ///
    /// Panics if the tokio runtime cannot be created or if the Selenium container fails to start.
    pub fn new(init: impl Future<Output = Doco>) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");

        let doco = rt.block_on(init);

        info!("initializing ephemeral test environment");

        let selenium = rt
            .block_on(Session::start_selenium())
            .expect("failed to initialize the test runner");

        Self {
            rt,
            doco,
            selenium: Arc::new(selenium),
        }
    }

    /// Collects all registered tests, wraps them as libtest-mimic trials, and runs them
    pub fn run(self) {
        let runner = Arc::new(self);
        let args = libtest_mimic::Arguments::from_args();

        let tests: Vec<libtest_mimic::Trial> = inventory::iter::<TestCase>
            .into_iter()
            .map(|tc| {
                let r = Arc::clone(&runner);
                let handle = runner.rt.handle().clone();
                let name = tc.name;
                let func = tc.function;
                libtest_mimic::Trial::test(tc.name, move || {
                    handle
                        .block_on(r.run_test(name, func))
                        .map_err(|e| e.into())
                })
            })
            .collect();

        libtest_mimic::run(&args, tests).exit();
    }

    /// Runs the given test in a clean, ephemeral environment
    ///
    /// Starts any auxiliary services like databases and waits for them to be ready, then starts
    /// the server, configures the WebDriver [`Client`], and calls the test function.
    ///
    /// # Errors
    ///
    /// Returns an error if any container fails to start, if the WebDriver connection fails, or
    /// if the test function itself returns an error.
    pub async fn run_test(&self, name: &str, test: fn(Client) -> Result<()>) -> Result<()> {
        let _span = info_span!("test", name).entered();
        let session = Session::with_selenium(&self.doco, Arc::clone(&self.selenium)).await?;
        let client = session.client().clone();

        test(client)?;

        session.close().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use axum::routing::get;
    use axum::Router;
    use tokio::net::TcpListener;

    use crate::test_utils::*;
    use crate::Result;

    use super::*;

    #[test]
    fn filter_selects_matching_tests() {
        let trials = vec![
            libtest_mimic::Trial::test("alpha_test", || Ok(())),
            libtest_mimic::Trial::test("beta_test", || Err("should not run".into())),
        ];

        let args = libtest_mimic::Arguments {
            filter: Some("alpha".into()),
            ..Default::default()
        };

        let conclusion = libtest_mimic::run(&args, trials);

        assert_eq!(conclusion.num_passed, 1);
        assert_eq!(conclusion.num_failed, 0);
        assert_eq!(conclusion.num_filtered_out, 1);
    }

    #[tokio::test]
    async fn headless_browser_can_navigate() -> Result<()> {
        let listener = TcpListener::bind("0.0.0.0:0").await?;
        let port = listener.local_addr()?.port();

        let app = Router::new().route("/", get(|| async { "headless works" }));
        tokio::spawn(async { axum::serve(listener, app).await });

        let selenium = Session::start_selenium().await?;

        let mut caps = thirtyfour::DesiredCapabilities::firefox();
        caps.set_headless()?;

        let driver = thirtyfour::WebDriver::new(
            &format!(
                "http://{}:{}",
                selenium.get_host().await?,
                selenium.get_host_port_ipv4(4444).await?
            ),
            caps,
        )
        .await
        .expect("failed to connect to headless WebDriver");

        driver
            .goto(&format!("http://host.docker.internal:{port}/"))
            .await?;
        let body = driver.source().await?;

        assert!(body.contains("headless works"));

        driver.quit().await.ok();

        Ok(())
    }

    #[test]
    fn list_flag_prints_test_names() {
        let trials = vec![
            libtest_mimic::Trial::test("alpha_test", || Ok(())),
            libtest_mimic::Trial::test("beta_test", || Ok(())),
        ];

        let args = libtest_mimic::Arguments {
            list: true,
            ..Default::default()
        };

        let conclusion = libtest_mimic::run(&args, trials);

        // --list exits without running anything, so no tests pass or fail
        assert_eq!(conclusion.num_passed, 0);
        assert_eq!(conclusion.num_failed, 0);
    }

    #[tokio::test]
    async fn selenium_can_access_host() -> Result<()> {
        let listener = TcpListener::bind("0.0.0.0:0").await?;
        let port = listener.local_addr()?.port();

        let app = Router::new().route("/", get(|| async { "hello from the test" }));
        tokio::spawn(async { axum::serve(listener, app).await });

        let selenium = Session::start_selenium().await?;

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
            .goto(&format!("http://host.docker.internal:{port}/"))
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

    #[tokio::test]
    async fn viewport_sets_window_dimensions() -> Result<()> {
        let selenium = Session::start_selenium().await?;

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

        let viewport = crate::Viewport::new(1280, 720);
        driver
            .set_window_rect(0, 0, viewport.width(), viewport.height())
            .await?;

        let inner_width: u64 = driver
            .execute("return window.innerWidth", vec![])
            .await?
            .json()
            .as_u64()
            .unwrap();
        let inner_height: u64 = driver
            .execute("return window.innerHeight", vec![])
            .await?
            .json()
            .as_u64()
            .unwrap();

        // Window chrome takes some space, so innerWidth/innerHeight may differ slightly from the
        // outer rect. But set_window_rect sets the outer dimensions, so inner dimensions should be
        // close. The key assertion is that they changed from the default.
        assert!(inner_width > 0, "innerWidth should be positive");
        assert!(inner_height > 0, "innerHeight should be positive");

        // The outer rect should match exactly what we requested
        let rect = driver.get_window_rect().await?;
        assert_eq!(rect.width, 1280);
        assert_eq!(rect.height, 720);

        driver.quit().await.ok();

        Ok(())
    }
}
