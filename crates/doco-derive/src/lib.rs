//! Derive macros for the Doco testing framework
//!
//! Doco is a test runner and library for writing end-to-tests of web applications. It runs tests
//! in isolated, ephemeral environments. This crate provides procedural macros to make it easier to
//! set up the test runner, collect all tests, and then run them individually in isolated, ephemeral
//! environments.
//!
//! It is not recommended to use this crate directly. Instead, use the [`doco`] crate that
//! re-exports the macros from this crate.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

/// Collect and run the end-to-end tests with Doco
///
/// This macro makes it very easy to use the [`doco`] testing framework. It collects all tests that
/// are annotated with the [`doco::test`] macro, initializes the test runner, and then runs each
/// test in an isolated, ephemeral environment.
///
/// # Examples
///
/// ```ignore
/// use doco::{Doco, Server};
///
/// #[doco::main]
/// async fn main() -> Doco {
///     let server = Server::builder()
///         .image("crccheck/hello-world")
///         .tag("v1.0.0")
///         .port(8000)
///         .build();
///
///     Doco::builder().server(server).build()
/// }
/// ```
#[proc_macro_attribute]
pub fn main(_args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the function that has been annotated with the `#[doco_derive::main]` attribute
    let main_fn = parse_macro_input!(input as ItemFn);
    let main_block = main_fn.block;

    let expanded = quote! {
        fn main() {
            doco::TestRunner::new(async #main_block).run();
        }
    };

    expanded.into()
}

/// Annotate an end-to-end test to be run with Doco
///
/// The `#[doco::test]` attribute is used to annotate an asynchronous test function that should be
/// executed by Doco as an end-to-end test. The test function is passed a [`doco::Client`] that can
/// be used to interact with the web application, and it should return a [`doco::Result`].
///
/// # Examples
///
/// ```ignore
/// use doco::{Client, Result};
///
/// #[doco::test]
/// async fn visit_root_path(client: Client) -> Result<()> {
///     client.goto("/").await?;
///
///     let body = client.source().await?;
///
///     assert!(body.contains("Hello World"));
///
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn test(_attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the function that has been annotated with the `#[doco_derive::test]` attribute
    let input_fn = parse_macro_input!(input as ItemFn);
    let input_fn_ident = &input_fn.sig.ident;
    let input_fn_name = input_fn_ident.to_string();

    // Extract the function name, arguments, and body for the final test function
    let test_fn_ident = format_ident!("{}_test", &input_fn_ident);
    let test_args = &input_fn.sig.inputs;

    // Generate a test function that executes the test block inside doco's asynchronous runtime
    let test_function = quote! {
        #input_fn

        fn #test_fn_ident(#test_args) -> doco::Result<()> {
            std::thread::spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build()?;

                runtime.block_on(async {
                    #input_fn_ident(client).await
                })
            })
            .join().map_err(|_| doco::anyhow!("failed to run test in isolated thread"))?
        }

        doco::inventory::submit!(doco::TestCase {
            name: #input_fn_name,
            function: #test_fn_ident
        });
    };

    test_function.into()
}
