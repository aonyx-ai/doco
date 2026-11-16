//! Environment variables for services

use getset::Getters;
use typed_builder::TypedBuilder;

/// Environment variable for a service
///
/// The server and its optional services might require additional configuration to start
/// successfully. For example, the official Docker image for Postgres requires the
/// `POSTGRES_PASSWORD` environment variable to be set. This struct represents an environment
/// variable that can be passed to a service.
///
/// Environment variables are in implementation detail of the `doco` crate and are not exposed
/// publicly. Users can configure them by calling the `env` method on the `ServiceBuilder`. See the
/// [`Service`] struct for more information.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Getters, TypedBuilder)]
pub struct Variable {
    /// The name of the environment variable
    #[builder(setter(into))]
    #[getset(get = "pub")]
    name: String,

    /// The value of the environment variable
    #[builder(setter(into))]
    #[getset(get = "pub")]
    value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_auto_traits<T: Send + Sync + Unpin>() {}

    #[test]
    fn builder_with_str_args_stores_name() {
        let variable = Variable::builder().name("name").value("value").build();

        assert_eq!(variable.name(), "name");
    }

    #[test]
    fn builder_with_str_args_stores_value() {
        let variable = Variable::builder().name("name").value("value").build();

        assert_eq!(variable.value(), "value");
    }

    #[test]
    fn builder_with_string_args_stores_name() {
        let name = String::from("name");
        let value = String::from("value");

        let variable = Variable::builder().name(name).value(value).build();

        assert_eq!(variable.name(), "name");
    }

    #[test]
    fn builder_with_string_args_stores_value() {
        let name = String::from("name");
        let value = String::from("value");

        let variable = Variable::builder().name(name).value(value).build();

        assert_eq!(variable.value(), "value");
    }

    #[test]
    fn variable_is_send_sync_and_unpin() {
        assert_auto_traits::<Variable>();
    }
}
