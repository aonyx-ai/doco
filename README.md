# 🦕 Doco

Doco is a framework and runner for end-to-end tests of web applications. It is
designed to be framework-agnostic and easy to use, both locally and in CI/CD
pipelines.

Tests are run in isolated, ephemeral environments using Docker containers. Doco
hides the complexities of setting up the test environment, configuring Selenium,
and managing the lifecycle of the containers.

## Usage

Doco looks and feels mostly like any other test framework in Rust. It provides
two macros, `#[doco::test]` and `#[doco::main]`, to define tests and configure
the test runner.

Start by adding Doco to your `Cargo.toml` as a custom test harness:

```toml
[[test]]
name = "e2e"
path = "e2e/main.rs"
harness = false
```

Then go ahead and create the `e2e` directory in the same directory as your
`Cargo.toml` and add a `main.rs` file. In this file, add the following snippet
and customize the server with your own Docker `image` and `tag` and the right
`port`:

```rust
use doco::{Doco, Server};

#[doco::main]
async fn main() -> Doco {
    let server = Server::builder()
        .image("image")
        .tag("tag")
        .port(3000)
        .build();

    Doco::builder().server(server).build()
}
```

Now you can write your first test. Either in a new file or in `main.rs`, add an
asynchronous function that takes a `Client` as its argument and returns a
`Result<()>`.

```rust
use doco::{Client, Result};

#[doco::test]
async fn reads_from_database(client: Client) -> Result<()> {
    client.goto("/").await?;

    let body = client.source().await?;

    assert!(body.contains("hello world from Doco"));

    Ok(())
}
```

Make sure to read the [API documentation](https://docs.rs/doco) for more
information, and check out the [examples](examples) directory for more examples.

## Limitations

Doco is still in an early stage of development and has some known limitations.
These will be addressed in future releases.

- Rust is the only supported language for writing tests.
- Doco only supports a fail-fast mode and stops after the first failing test.
- Each test must have a globally unique name.
- Errors from Docker are not reported properly yet.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
