//! Browser display mode for the tests

/// Whether the browser runs with or without a visible window
///
/// The default follows the environment: the browser runs headless when the `CI` environment
/// variable is set, and headed otherwise.
///
/// # Examples
///
/// ```rust
/// use doco::{Doco, Headless, Server};
///
/// let server = Server::builder().image("app").tag("latest").port(8000).build();
///
/// let doco = Doco::builder()
///     .server(server)
///     .headless(Headless::Enabled)
///     .build();
///
/// assert_eq!(doco.headless(), Headless::Enabled);
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Headless {
    /// Firefox runs without a visible window
    Enabled,

    /// Firefox runs with a visible window
    Disabled,
}

impl Default for Headless {
    fn default() -> Self {
        match std::env::var("CI") {
            Ok(_) => Headless::Enabled,
            Err(_) => Headless::Disabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_auto_traits<T: Send + Sync + Unpin>() {}

    #[test]
    fn headless_is_send_sync_and_unpin() {
        assert_auto_traits::<Headless>();
    }
}
