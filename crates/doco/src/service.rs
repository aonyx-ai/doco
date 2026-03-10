//! Auxiliary service required by the server

use getset::{CopyGetters, Getters};
use testcontainers::core::{Mount, WaitFor};
use typed_builder::TypedBuilder;

use crate::environment::Variable;

/// Auxiliary service required by the server
///
/// The [`Server`] might require additional services to work, e.g. a database. These can be
/// defined using the `Service` struct and added to the [`Doco`] configuration. Each service is run
/// as a Docker container and can be configured with environment variables.
///
/// Services can be accessed from the server by using the `image` name. For example, adding a
/// `postgres` service will allow the server to connect to `postgres:5432`. See the `axum-postgres`
/// example in the repository for a working demo.
///
/// # Examples
///
/// ```rust
/// use doco::Service;
///
/// fn postgres() -> Service {
///     Service::builder()
///         .image("postgres")
///         .tag("latest")
///         .env("POSTGRES_PASSWORD", "password")
///         .build()
/// }
/// ```
#[derive(Clone, Debug, Default, CopyGetters, Getters, TypedBuilder)]
pub struct Service {
    /// The name of the service's Docker image
    #[builder(setter(into))]
    #[getset(get = "pub")]
    image: String,

    /// The tag of the service's Docker image
    #[builder(setter(into))]
    #[getset(get = "pub")]
    tag: String,

    /// Environment variables to set in the service's container
    #[builder(via_mutators(init = Vec::new()), mutators(
        pub fn env(mut self, name: impl Into<String>, value: impl Into<String>) {
            self.envs.push(Variable::new(name, value));
        }
    ))]
    #[getset(get = "pub")]
    envs: Vec<Variable>,

    /// Filesystem mounts for the service's container
    #[builder(via_mutators(init = Vec::new()), mutators(
        /// Adds a filesystem mount to the service's container
        pub fn mount(mut self, mount: Mount) {
            self.mounts.push(mount);
        }
    ))]
    #[getset(get = "pub")]
    mounts: Vec<Mount>,

    /// Arguments for the container's command, overriding the image default
    #[builder(via_mutators(init = Vec::new()), mutators(
        /// Adds an argument to the container's command
        pub fn cmd_arg(mut self, arg: impl Into<String>) {
            self.cmd.push(arg.into());
        }
    ))]
    #[getset(get = "pub")]
    cmd: Vec<String>,

    /// An optional condition to wait until the service has properly started
    #[builder(default, setter(into))]
    #[getset(get = "pub")]
    wait: Option<WaitFor>,
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;

    use super::*;

    #[test]
    fn cmd_arg_collects_args() {
        let service = Service::builder()
            .image("postgres")
            .tag("latest")
            .cmd_arg("--config")
            .cmd_arg("/etc/postgres.conf")
            .build();

        assert_eq!(2, service.cmd.len());
    }

    #[test]
    fn env_collects_variables() {
        let service = Service::builder()
            .image("postgres")
            .tag("latest")
            .env("POSTGRES_PASSWORD", "password")
            .env("POSTGRES_USER", "postgres")
            .env("POSTGRES_PASSWORD", "postgres")
            .build();

        assert_eq!(3, service.envs.len());
    }

    #[test]
    fn mount_collects_mounts() {
        let service = Service::builder()
            .image("postgres")
            .tag("latest")
            .mount(Mount::bind_mount("/host/data", "/var/lib/postgresql/data"))
            .mount(Mount::bind_mount("/host/config", "/etc/postgresql"))
            .build();

        assert_eq!(2, service.mounts.len());
    }

    #[test]
    fn trait_send() {
        assert_send::<Service>();
    }

    #[test]
    fn trait_sync() {
        assert_sync::<Service>();
    }

    #[test]
    fn trait_unpin() {
        assert_unpin::<Service>();
    }
}
