//! Viewport configuration for browser window size

/// Browser viewport dimensions
///
/// Controls the browser window size for tests. This is particularly important for visual
/// regression testing where screenshots must be captured at a consistent size.
///
/// # Examples
///
/// ```rust
/// use doco::{Doco, Server, Viewport};
///
/// # #[doco::main]
/// # async fn main() -> Doco {
/// let server = Server::builder()
///     .image("my-app")
///     .tag("latest")
///     .port(8080)
///     .build();
///
/// Doco::builder()
///     .server(server)
///     .viewport(Viewport::new(1280, 720))
///     .build()
/// # }
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Viewport {
    /// The width of the browser window in pixels
    width: u32,

    /// The height of the browser window in pixels
    height: u32,
}

impl Viewport {
    /// Create a new viewport with the given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// The width of the browser window in pixels
    pub fn width(&self) -> u32 {
        self.width
    }

    /// The height of the browser window in pixels
    pub fn height(&self) -> u32 {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;

    use super::*;

    #[test]
    fn new_stores_dimensions() {
        let vp = Viewport::new(1280, 720);
        assert_eq!(vp.width(), 1280);
        assert_eq!(vp.height(), 720);
    }

    #[test]
    fn trait_send() {
        assert_send::<Viewport>();
    }

    #[test]
    fn trait_sync() {
        assert_sync::<Viewport>();
    }

    #[test]
    fn trait_unpin() {
        assert_unpin::<Viewport>();
    }
}
