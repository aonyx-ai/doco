use std::time::Duration;

use doco::{By, Client, Doco, Result, Server};
use tokio::time::sleep;

#[doco::test]
async fn has_title(client: Client) -> Result<()> {
    client.goto("/").await?;

    let title = client
        .find(By::XPath("/html/body/main/h1"))
        .await?
        .text()
        .await?;

    assert_eq!("Welcome to Leptos!", title);

    Ok(())
}

#[doco::test]
async fn clicking_button_increases_counter(client: Client) -> Result<()> {
    client.goto("/").await?;

    let button = client.find(By::XPath("/html/body/main/button")).await?;

    let before = button.text().await?;
    assert_eq!("Click Me: 0", before);

    button.click().await?;

    // Wait for the button to update
    sleep(Duration::from_secs(1)).await;

    let after = button.text().await?;
    assert_eq!("Click Me: 1", after);

    Ok(())
}

#[doco::main]
async fn main() -> Doco {
    let server = Server::builder()
        .image("doco")
        .tag("leptos")
        .port(8080)
        .build();

    Doco::builder().server(server).build()
}
