#![allow(missing_docs)]

#[global_allocator]
static ALLOC: reth_ethereum::cli::allocator::Allocator =
    reth_ethereum::cli::allocator::new_allocator();
use clap::Parser;
use tracing::info;

mod cli;
use cli::Cli;

mod node;
use node::CustomNode;

mod chainspec;

fn main() {
    reth_ethereum::cli::sigsegv_handler::install();

    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    if let Err(err) = Cli::parse().run(async move |builder, _| {
        info!(target: "reth::cli", "Launching node");
        let handle = builder.node(CustomNode::default()).launch_with_debug_capabilities().await?;

        handle.node_exit_future.await
    }) {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
