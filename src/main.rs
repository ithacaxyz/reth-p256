#![allow(missing_docs)]

#[global_allocator]
static ALLOC: reth_ethereum::cli::allocator::Allocator =
    reth_ethereum::cli::allocator::new_allocator();
use clap::Parser;
use reth_ethereum::{
    cli::{Cli, chainspec::EthereumChainSpecParser},
    node::builder::NodeHandle,
};
use tracing::info;

mod node;
use node::CustomNode;

fn main() {
    reth_ethereum::cli::sigsegv_handler::install();

    // Enable backtraces unless a RUST_BACKTRACE value has already been explicitly provided.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    if let Err(err) = Cli::<EthereumChainSpecParser>::parse().run(async move |builder, _| {
        info!(target: "reth::cli", "Launching node");
        let NodeHandle { node_exit_future, .. } =
            builder.node(CustomNode::default()).launch_with_debug_capabilities().await?;

        node_exit_future.await
    }) {
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
