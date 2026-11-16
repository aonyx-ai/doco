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
///     .viewport(Viewport::builder().width(1280).height(720).build())
///     .build()
/// # }
/// ```
#[derive(Copy, Clone, Eq, PartialEq, Debug, bon::Builder)]
pub struct Viewport {
    /// The width of the browser window in pixels
    width: u32,

    /// The height of the browser window in pixels
    height: u32,
}

impl Viewport {
    /// Returns the width of the browser window in pixels
    ///
    /// # Examples
    ///
    /// ```rust
    /// use doco::Viewport;
    ///
    /// let viewport = Viewport::builder().width(1280).height(720).build();
    ///
    /// assert_eq!(viewport.width(), 1280);
    /// ```
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height of the browser window in pixels
    ///
    /// # Examples
    ///
    /// ```rust
    /// use doco::Viewport;
    ///
    /// let viewport = Viewport::builder().width(1280).height(720).build();
    ///
    /// assert_eq!(viewport.height(), 720);
    /// ```
    pub fn height(&self) -> u32 {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_auto_traits<T: Send + Sync + Unpin>() {}

    #[test]
    fn builder_stores_height() {
        let viewport = Viewport::builder().width(1280).height(720).build();

        assert_eq!(viewport.height(), 720);
    }

    #[test]
    fn builder_stores_width() {
        let viewport = Viewport::builder().width(1280).height(720).build();

        assert_eq!(viewport.width(), 1280);
    }

    #[test]
    fn viewport_is_send_sync_and_unpin() {
        assert_auto_traits::<Viewport>();
    }
}
