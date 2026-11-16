use std::process::{Command, Output};
use std::sync::OnceLock;

fn fixture_output() -> &'static Output {
    static OUTPUT: OnceLock<Output> = OnceLock::new();

    OUTPUT.get_or_init(|| {
        Command::new("cargo")
            .args(["run", "--example", "tracing_fixture"])
            .env("RUST_LOG", "doco=debug")
            .env("CI", "1")
            .output()
            .expect("failed to run tracing_fixture example")
    })
}

#[test]
fn tracing_fixture_exits_successfully() {
    let output = fixture_output();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "tracing_fixture exited with failure:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        stderr,
    );
}

#[test]
fn tracing_output_contains_lifecycle_messages() {
    let output = fixture_output();
    let stderr = String::from_utf8_lossy(&output.stderr);

    let expected = [
        "initializing ephemeral test environment",
        "starting selenium container",
        "starting server container",
        "server container ready",
        "connecting to WebDriver",
        "navigating",
        "closing session",
    ];

    for message in expected {
        assert!(
            stderr.contains(message),
            "expected stderr to contain {message:?}, but it was not found.\nfull stderr:\n{stderr}",
        );
    }
}
