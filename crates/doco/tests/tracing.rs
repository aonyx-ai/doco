use std::process::Command;

#[test]
fn tracing_output_contains_lifecycle_messages() {
    let output = Command::new("cargo")
        .args(["run", "--example", "tracing_fixture"])
        .env("RUST_LOG", "doco=debug")
        .env("CI", "1")
        .output()
        .expect("failed to run tracing_fixture example");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "tracing_fixture exited with failure:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        stderr,
    );

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
