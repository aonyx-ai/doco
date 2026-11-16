//! Errors that Doco returns from fallible actions

use testcontainers::TestcontainersError;
use thirtyfour::error::WebDriverError;

/// The reasons why Doco fails to start a browser session
///
/// Each variant names the step that failed and holds the underlying error as its source.
///
/// # Examples
///
/// ```rust
/// use doco::ConnectError;
///
/// fn is_selenium_failure(error: &ConnectError) -> bool {
///     if let ConnectError::StartSelenium { .. } = error {
///         return true;
///     }
///
///     false
/// }
/// ```
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ConnectError {
    /// The Selenium container did not start
    #[error("failed to start the Selenium container")]
    StartSelenium {
        /// The error that the container runtime returned
        source: TestcontainersError,
    },

    /// A service container did not start
    #[error("failed to start the service container {image}")]
    StartService {
        /// The image of the service
        image: String,

        /// The error that the container runtime returned
        source: TestcontainersError,
    },

    /// The bridge address of a service container is unknown
    #[error("failed to resolve the bridge address of the service {image}")]
    ResolveServiceAddress {
        /// The image of the service
        image: String,

        /// The error that the container runtime returned
        source: TestcontainersError,
    },

    /// The server container did not start
    #[error("failed to start the server container")]
    StartServer {
        /// The error that the container runtime returned
        source: TestcontainersError,
    },

    /// The host port of the server container is unknown
    #[error("failed to resolve the host port of the server container")]
    ResolveServerPort {
        /// The error that the container runtime returned
        source: TestcontainersError,
    },

    /// The base URL of the server is not a valid URL
    #[error("failed to parse the base URL of the server")]
    ParseServerUrl {
        /// The error that the URL parser returned
        source: url::ParseError,
    },

    /// The address of the Selenium container is unknown
    #[error("failed to resolve the address of the Selenium container")]
    ResolveSeleniumEndpoint {
        /// The error that the container runtime returned
        source: TestcontainersError,
    },

    /// The headless capability could not be set
    #[error("failed to set the headless capability")]
    SetHeadless {
        /// The error that the WebDriver library returned
        source: WebDriverError,
    },

    /// A Firefox preference could not be set
    #[error("failed to set the Firefox preference {name}")]
    SetPreference {
        /// The name of the preference
        name: String,

        /// The error that the WebDriver library returned
        source: WebDriverError,
    },

    /// The Firefox preferences could not be applied
    #[error("failed to apply the Firefox preferences")]
    ApplyPreferences {
        /// The error that the WebDriver library returned
        source: WebDriverError,
    },

    /// The WebDriver connection could not be established
    #[error("failed to connect to the WebDriver")]
    ConnectWebDriver {
        /// The error that the WebDriver library returned
        source: WebDriverError,
    },

    /// The browser window could not be resized
    #[error("failed to set the browser viewport")]
    SetViewport {
        /// The error that the WebDriver library returned
        source: WebDriverError,
    },
}

/// The reasons why Doco fails to run a test
///
/// # Examples
///
/// ```rust
/// use doco::RunTestError;
///
/// fn is_test_failure(error: &RunTestError) -> bool {
///     match error {
///         RunTestError::RunTest { .. } => true,
///         RunTestError::StartSession { .. } => false,
///     }
/// }
/// ```
#[derive(Debug, thiserror::Error)]
pub enum RunTestError {
    /// The environment for the test did not start
    #[error("failed to start the session for the test {name}: {source}")]
    StartSession {
        /// The name of the test
        name: String,

        /// The error that stopped the session from starting
        source: Box<ConnectError>,
    },

    /// The test function returned an error
    #[error("failed to run the test {name}: {source}")]
    RunTest {
        /// The name of the test
        name: String,

        /// The error that the test function returned
        source: anyhow::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_auto_traits<T: Send + Sync + Unpin>() {}

    #[test]
    fn connect_error_is_send_sync_and_unpin() {
        assert_auto_traits::<ConnectError>();
    }

    #[test]
    fn run_test_error_is_send_sync_and_unpin() {
        assert_auto_traits::<RunTestError>();
    }
}
