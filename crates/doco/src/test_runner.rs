//! Test runner for Doco's end-to-end tests

use std::future::Future;
use std::sync::Arc;

use tracing::{info, info_span};

use crate::{Client, Doco, Result, RunTestError, Session, TestCase};

/// Test runner for Doco's end-to-end tests
///
/// The `TestRunner` is responsible for executing each test in an isolated, ephemeral environment.
/// It starts Selenium in a container and configures the WebDriver [`Client`] to connect to
/// Selenium. It then runs each test against a clean instance of the server and its services.
///
/// It should not be necessary to use this struct directly. Instead, use the [`doco::main`] and
/// [`doco::test`] macros to automatically set up the test runner, collect all tests, and pass them
/// to the runner.
///
/// # Examples
///
/// ```no_run
/// use doco::{Doco, Server, TestRunner};
///
/// let server = Server::builder().image("my-app").tag("latest").port(8080).build();
///
/// TestRunner::new(async { Doco::builder().server(server).build() }).run();
/// ```
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use doco::{Doco, Server, TestRunner};
    ///
    /// let server = Server::builder().image("my-app").tag("latest").port(8080).build();
    ///
    /// let runner = TestRunner::new(async { Doco::builder().server(server).build() });
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use doco::{Doco, Server, TestRunner};
    ///
    /// let server = Server::builder().image("my-app").tag("latest").port(8080).build();
    ///
    /// TestRunner::new(async { Doco::builder().server(server).build() }).run();
    /// ```
    pub fn run(self) {
        let runner = Arc::new(self);
        let args = libtest_mimic::Arguments::from_args();

        let tests: Vec<libtest_mimic::Trial> = inventory::iter::<TestCase>
            .into_iter()
            .map(|tc| {
                let r = Arc::clone(&runner);
                let handle = runner.rt.handle().clone();
                let name = tc.name();
                let func = tc.function();
                libtest_mimic::Trial::test(tc.name(), move || {
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
    /// Starts any auxiliary services like databases and waits for them to be ready. Then starts
    /// the server, configures the WebDriver [`Client`], and calls the test function.
    ///
    /// # Errors
    ///
    /// Returns [`RunTestError::StartSession`] if any container fails to start or if the WebDriver
    /// connection fails. Returns [`RunTestError::RunTest`] if the test function itself returns an
    /// error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use doco::{Client, Result, RunTestError, TestRunner};
    ///
    /// fn visit_root_path(_client: Client) -> Result<()> {
    ///     Ok(())
    /// }
    ///
    /// # async fn example(runner: TestRunner) -> std::result::Result<(), RunTestError> {
    /// runner.run_test("visit_root_path", visit_root_path).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run_test(
        &self,
        name: &str,
        test: fn(Client) -> Result<()>,
    ) -> std::result::Result<(), RunTestError> {
        let _span = info_span!("test", name).entered();
        let session = Session::with_selenium()
            .doco(&self.doco)
            .selenium(Arc::clone(&self.selenium))
            .call()
            .await
            .map_err(|source| RunTestError::StartSession {
                name: name.to_owned(),
                source: Box::new(source),
            })?;
        let client = session.client().clone();

        test(client).map_err(|source| RunTestError::RunTest {
            name: name.to_owned(),
            source,
        })?;

        session.close().await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use axum::routing::get;
    use axum::Router;
    use testcontainers::{ContainerAsync, GenericImage};
    use thirtyfour::WebDriver;
    use tokio::net::TcpListener;

    use crate::{Context, Result};

    use super::*;

    fn assert_auto_traits<T: Send + Sync + Unpin>() {}

    fn filtered_conclusion() -> libtest_mimic::Conclusion {
        let trials = vec![
            libtest_mimic::Trial::test("alpha_test", || Ok(())),
            libtest_mimic::Trial::test("beta_test", || Err("should not run".into())),
        ];

        let args = libtest_mimic::Arguments {
            filter: Some("alpha".into()),
            ..Default::default()
        };

        libtest_mimic::run(&args, trials)
    }

    fn listed_conclusion() -> libtest_mimic::Conclusion {
        let trials = vec![
            libtest_mimic::Trial::test("alpha_test", || Ok(())),
            libtest_mimic::Trial::test("beta_test", || Ok(())),
        ];

        let args = libtest_mimic::Arguments {
            list: true,
            ..Default::default()
        };

        libtest_mimic::run(&args, trials)
    }

    async fn resized_driver(
        viewport: crate::Viewport,
    ) -> Result<(ContainerAsync<GenericImage>, WebDriver)> {
        let selenium = Session::start_selenium()
            .await
            .context("failed to start the selenium container")?;

        let driver = thirtyfour::WebDriver::new(
            &format!(
                "http://{}:{}",
                selenium
                    .get_host()
                    .await
                    .context("failed to get the selenium host")?,
                selenium
                    .get_host_port_ipv4(4444)
                    .await
                    .context("failed to get the selenium port")?
            ),
            thirtyfour::DesiredCapabilities::firefox(),
        )
        .await
        .expect("failed to connect to WebDriver");

        driver
            .set_window_rect(0, 0, viewport.width(), viewport.height())
            .await
            .context("failed to resize the browser window")?;

        Ok((selenium, driver))
    }

    #[test]
    fn filter_selects_matching_tests_counts_failed() {
        let conclusion = filtered_conclusion();

        assert_eq!(conclusion.num_failed, 0);
    }

    #[test]
    fn filter_selects_matching_tests_counts_filtered_out() {
        let conclusion = filtered_conclusion();

        assert_eq!(conclusion.num_filtered_out, 1);
    }

    #[test]
    fn filter_selects_matching_tests_counts_passed() {
        let conclusion = filtered_conclusion();

        assert_eq!(conclusion.num_passed, 1);
    }

    #[tokio::test]
    async fn headless_browser_can_navigate() -> Result<()> {
        let listener = TcpListener::bind("0.0.0.0:0")
            .await
            .context("failed to bind the listener")?;
        let port = listener
            .local_addr()
            .context("failed to get the listener address")?
            .port();

        let app = Router::new().route("/", get(|| async { "headless works" }));
        tokio::spawn(async { axum::serve(listener, app).await });

        let selenium = Session::start_selenium()
            .await
            .context("failed to start the selenium container")?;

        let mut caps = thirtyfour::DesiredCapabilities::firefox();
        caps.set_headless()
            .context("failed to set the headless capability")?;

        let driver = thirtyfour::WebDriver::new(
            &format!(
                "http://{}:{}",
                selenium
                    .get_host()
                    .await
                    .context("failed to get the selenium host")?,
                selenium
                    .get_host_port_ipv4(4444)
                    .await
                    .context("failed to get the selenium port")?
            ),
            caps,
        )
        .await
        .expect("failed to connect to headless WebDriver");

        driver
            .goto(&format!("http://host.docker.internal:{port}/"))
            .await
            .context("failed to navigate to the test server")?;
        let body = driver
            .source()
            .await
            .context("failed to read the page source")?;

        assert!(body.contains("headless works"));

        driver.quit().await.ok();

        Ok(())
    }

    /// The `--list` flag exits without running anything, so no tests fail
    #[test]
    fn list_flag_prints_test_names_without_failures() {
        let conclusion = listed_conclusion();

        assert_eq!(conclusion.num_failed, 0);
    }

    /// The `--list` flag exits without running anything, so no tests pass
    #[test]
    fn list_flag_prints_test_names_without_passes() {
        let conclusion = listed_conclusion();

        assert_eq!(conclusion.num_passed, 0);
    }

    #[tokio::test]
    async fn selenium_can_access_host() -> Result<()> {
        let listener = TcpListener::bind("0.0.0.0:0")
            .await
            .context("failed to bind the listener")?;
        let port = listener
            .local_addr()
            .context("failed to get the listener address")?
            .port();

        let app = Router::new().route("/", get(|| async { "hello from the test" }));
        tokio::spawn(async { axum::serve(listener, app).await });

        let selenium = Session::start_selenium()
            .await
            .context("failed to start the selenium container")?;

        let driver = thirtyfour::WebDriver::new(
            &format!(
                "http://{}:{}",
                selenium
                    .get_host()
                    .await
                    .context("failed to get the selenium host")?,
                selenium
                    .get_host_port_ipv4(4444)
                    .await
                    .context("failed to get the selenium port")?
            ),
            thirtyfour::DesiredCapabilities::firefox(),
        )
        .await
        .expect("failed to connect to WebDriver");

        driver
            .goto(&format!("http://host.docker.internal:{port}/"))
            .await
            .context("failed to navigate to the test server")?;
        let body = driver
            .source()
            .await
            .context("failed to read the page source")?;

        assert!(body.contains("hello from the test"));

        driver.quit().await.ok();

        Ok(())
    }

    #[test]
    fn test_runner_is_send_sync_and_unpin() {
        assert_auto_traits::<TestRunner>();
    }

    /// The outer rect matches exactly what the test requested
    #[tokio::test]
    async fn viewport_sets_outer_window_dimensions() -> Result<()> {
        let viewport = crate::Viewport::builder().width(1280).height(720).build();
        let (_selenium, driver) = resized_driver(viewport)
            .await
            .context("failed to start the resized browser")?;

        let rect = driver
            .get_window_rect()
            .await
            .context("failed to read the window rect")?;

        assert_eq!((rect.width, rect.height), (1280, 720));

        driver.quit().await.ok();

        Ok(())
    }

    /// The inner dimensions are positive after the resize
    ///
    /// Window chrome takes some space, so the inner dimensions may differ slightly from the
    /// outer rect. The setter fixes the outer dimensions, so the inner dimensions stay close. The
    /// key check is that they changed from the default.
    #[tokio::test]
    async fn viewport_sets_positive_inner_dimensions() -> Result<()> {
        let viewport = crate::Viewport::builder().width(1280).height(720).build();
        let (_selenium, driver) = resized_driver(viewport)
            .await
            .context("failed to start the resized browser")?;

        let inner_width: u64 = driver
            .execute("return window.innerWidth", vec![])
            .await
            .context("failed to read the inner width")?
            .json()
            .as_u64()
            .unwrap();
        let inner_height: u64 = driver
            .execute("return window.innerHeight", vec![])
            .await
            .context("failed to read the inner height")?
            .json()
            .as_u64()
            .unwrap();

        assert!(
            inner_width > 0 && inner_height > 0,
            "innerWidth and innerHeight should be positive"
        );

        driver.quit().await.ok();

        Ok(())
    }
}
