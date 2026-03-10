//! 🦕 Doco
//!
//! Doco is a test runner and library for writing end-to-tests of web applications. It is designed
//! to be framework-agnostic and easy to use, both locally and in CI/CD pipelines.
//!
//! Under the hood, Doco uses containers to create ephemeral, isolated environments for each test.
//! This prevents state to leak between tests and ensures that each test is run with a known and
//! predictable environment.
//!
//! Doco has a very simple, yet powerful API to make it easy to write tests. In a `main` function,
//! the environment for tests is defined and configured. Most importantly, Doco is told about the
//! server and its dependencies. Then, tests are written just like with any other Rust test. The
//! tests are passed a `Client` that can be used to interact with a website, making it easy to
//! simulate user interactions and write assertions against the web application.
//!
//! # Examples
//!
//! ```rust
//! use doco::{Client, Doco, Result, Server, Service, WaitFor};
//!
//! #[doco::test]
//! async fn visit_root_path(client: Client) -> Result<()> {
//!     client.goto("/").await?;
//!
//!     let body = client.source().await?;
//!
//!     assert!(body.contains("Hello World"));
//!
//!     Ok(())
//! }
//!
//! #[doco::main]
//! async fn main() -> Doco {
//!     let server = Server::builder()
//!         .image("crccheck/hello-world")
//!         .tag("v1.0.0")
//!         .port(8000)
//!         .build();
//!
//!     Doco::builder().server(server).build()
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

pub use anyhow::{anyhow, Context, Error, Result};
pub use doco_derive::{main, test};
use getset::Getters;
#[doc(hidden)]
pub use inventory;
pub use testcontainers::core::{AccessMode, Mount, WaitFor};
pub use thirtyfour::By;
use typed_builder::TypedBuilder;

pub use crate::client::Client;
pub use crate::server::Server;
pub use crate::service::Service;
pub use crate::session::Session;
pub use crate::test_runner::TestRunner;
pub use crate::tracing_init::init_tracing;
pub use crate::viewport::Viewport;

mod client;
mod environment;
mod server;
mod service;
mod session;
mod test_runner;
mod tracing_init;
mod viewport;

#[cfg(test)]
mod test_utils;

/// A test case registered by the `#[doco::test]` macro
///
/// Each test case consists of a name and a function that receives a [`Client`] and returns a
/// [`Result`]. Test cases are collected at link time using the [`inventory`] crate and run by the
/// [`TestRunner`].
pub struct TestCase {
    /// The name of the test
    pub name: &'static str,

    /// The test function to execute
    pub function: fn(Client) -> Result<()>,
}

inventory::collect!(TestCase);

/// Configuration for end-to-end tests with Doco
///
/// The `Doco` struct configures the environment that is used to run each test, most importantly the
/// application server and any additional services that it depends on. An instance of this struct
/// must be returned by the `main` function of the test suite.
///
/// # Examples
///
/// ```rust
/// use doco::{Doco, Server};
///
/// #[doco::main]
/// async fn main() -> Doco {
///     let server = Server::builder()
///         .image("crccheck/hello-world")
///         .tag("v1.0.0")
///         .port(8000)
///         .build();
///
///     Doco::builder().server(server).build()
/// }
/// ```
#[derive(Clone, Debug, Getters, TypedBuilder)]
pub struct Doco {
    /// The server that Doco will test
    #[getset(get = "pub")]
    server: Server,

    /// Additional services (e.g. databases or caches) that the server depends on
    #[builder(via_mutators(init = Vec::new()), mutators(
        pub fn service(mut self, service: Service) {
            self.services.push(service);
        }
    ))]
    #[getset(get = "pub")]
    services: Vec<Service>,

    /// Whether to run the browser in headless mode
    ///
    /// When `true`, Firefox runs without a visible window. Defaults to auto-detection: headless
    /// when the `CI` environment variable is set, headed otherwise.
    #[builder(default = std::env::var("CI").is_ok())]
    #[getset(get = "pub")]
    headless: bool,

    /// The browser viewport dimensions
    ///
    /// When set, the browser window is resized to these dimensions before each test runs. This
    /// ensures consistent rendering for visual regression testing.
    #[builder(default, setter(strip_option))]
    #[getset(get = "pub")]
    viewport: Option<Viewport>,
}

impl Doco {
    /// Connects to a long-lived browser session
    ///
    /// Starts the Selenium browser, the application server, and any configured services in Docker
    /// containers, then returns a [`Session`] with a ready-to-use [`Client`]. The session keeps
    /// all containers alive until it is dropped or [`Session::close()`] is called.
    ///
    /// Unlike the test runner, which creates a fresh environment per test, `connect()` creates a
    /// single session that can be reused across many operations. This is useful for scenarios like
    /// visual regression testing where you want to visit many pages without the overhead of
    /// restarting containers each time.
    ///
    /// # Errors
    ///
    /// Returns an error if any container fails to start or if the WebDriver connection cannot be
    /// established.
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
    pub async fn connect(&self) -> Result<Session> {
        Session::connect(self).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;

    use super::{Doco, Server, Service, TestCase};

    #[test]
    fn doco_service_collects_services() {
        let server = Server::builder()
            .image("crccheck/hello-world")
            .tag("v1.0.0")
            .port(8000)
            .build();

        let doco = Doco::builder()
            .server(server)
            .service(Service::builder().image("first").tag("latest").build())
            .service(Service::builder().image("second").tag("latest").build())
            .build();

        assert_eq!(doco.services().len(), 2);
    }

    #[test]
    fn doco_trait_send() {
        assert_send::<Doco>();
    }

    #[test]
    fn doco_trait_sync() {
        assert_sync::<Doco>();
    }

    #[test]
    fn doco_trait_unpin() {
        assert_unpin::<Doco>();
    }

    #[test]
    fn test_case_trait_send() {
        assert_send::<TestCase>();
    }

    #[test]
    fn test_case_trait_sync() {
        assert_sync::<TestCase>();
    }

    #[test]
    fn test_case_trait_unpin() {
        assert_unpin::<TestCase>();
    }
}
