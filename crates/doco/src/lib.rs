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
//! # Example
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
pub use testcontainers::core::WaitFor;
pub use thirtyfour::By;
use typed_builder::TypedBuilder;

pub use crate::client::Client;
pub use crate::server::Server;
pub use crate::service::Service;
pub use crate::test_runner::TestRunner;

mod client;
mod environment;
mod server;
mod service;
mod test_runner;

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
/// # Example
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
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;

    use super::{Doco, Server, Service};

    #[test]
    fn service_collects_services() {
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
    fn trait_send() {
        assert_send::<Doco>();
    }

    #[test]
    fn trait_sync() {
        assert_sync::<Doco>();
    }

    #[test]
    fn trait_unpin() {
        assert_unpin::<Doco>();
    }
}
