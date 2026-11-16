use doco::{Client, Context, Doco, Result, Server};

#[doco::test]
async fn visit_root(client: Client) -> Result<()> {
    client
        .goto("/")
        .await
        .context("failed to visit the root path")?;

    let body = client
        .source()
        .await
        .context("failed to read the page source")?;

    assert!(body.contains("Hello World"));

    Ok(())
}

#[doco::main]
async fn main() -> Doco {
    let server = Server::builder()
        .image("caddy")
        .tag("2-alpine")
        .port(80)
        .cmd_arg("caddy")
        .cmd_arg("respond")
        .cmd_arg("--listen")
        .cmd_arg(":80")
        .cmd_arg("Hello World")
        .build();

    Doco::builder().server(server).build()
}
