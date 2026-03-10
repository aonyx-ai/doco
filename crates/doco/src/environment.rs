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

impl Variable {
    /// Creates a new environment variable with the given name and value
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Variable {
        Variable {
            name: name.into(),
            value: value.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;

    use super::*;

    #[test]
    fn from_str() {
        let variable = Variable::new("name", "value");

        assert_eq!(variable.name(), "name");
        assert_eq!(variable.value(), "value");
    }

    #[test]
    fn from_string() {
        let name = String::from("name");
        let value = String::from("value");

        let variable = Variable::new(name, value);

        assert_eq!(variable.name(), "name");
        assert_eq!(variable.value(), "value");
    }

    #[test]
    fn trait_send() {
        assert_send::<Variable>();
    }

    #[test]
    fn trait_sync() {
        assert_sync::<Variable>();
    }

    #[test]
    fn trait_unpin() {
        assert_unpin::<Variable>();
    }
}
