//! Tracing initialization for Doco

use tracing_subscriber::EnvFilter;

/// Initializes the tracing subscriber with environment-based filtering
///
/// Sets up `tracing_subscriber::fmt` with an [`EnvFilter`] sourced from the `RUST_LOG`
/// environment variable. The default filter is `doco=info,warn`, which shows Doco's info-level
/// messages and warn-level messages from all other crates.
///
/// Uses `try_init()` internally so it is safe to call multiple times — subsequent calls are
/// silently ignored. This is important because test binaries may call `init_tracing()` from both
/// `#[doco::main]` and individual test setup.
///
/// # Examples
///
/// ```
/// doco::init_tracing();
/// ```
///
/// To see debug-level output from Doco, set the environment variable before running:
///
/// ```bash
/// RUST_LOG=doco=debug cargo run --example my_test
/// ```
///
/// [`EnvFilter`]: tracing_subscriber::EnvFilter
pub fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("doco=info,warn"));

    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .try_init()
        .ok();
}
