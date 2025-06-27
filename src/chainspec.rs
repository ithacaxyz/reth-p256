use std::sync::Arc;

use alloy_genesis::Genesis;
use reth_cli::chainspec::ChainSpecParser;
use reth_ethereum::{
    chainspec::{ChainSpec, EthChainSpec, EthereumHardforks, Hardfork, Hardforks},
    cli::chainspec::EthereumChainSpecParser,
    evm::{
        primitives::eth::spec::EthExecutorSpec,
        revm::primitives::{Address, B256, U256},
    },
    primitives::Header,
};
use reth_network_peers::NodeRecord;

#[derive(Debug, Clone)]
pub struct CustomChainSpec {
    inner: ChainSpec,
}

impl Hardforks for CustomChainSpec {
    fn fork<H: Hardfork>(&self, fork: H) -> reth_ethereum::chainspec::ForkCondition {
        self.inner.fork(fork)
    }

    fn forks_iter(
        &self,
    ) -> impl Iterator<Item = (&dyn Hardfork, reth_ethereum::chainspec::ForkCondition)> {
        self.inner.forks_iter()
    }

    fn fork_id(&self, head: &reth_ethereum::chainspec::Head) -> reth_ethereum::chainspec::ForkId {
        self.inner.fork_id(head)
    }

    fn latest_fork_id(&self) -> reth_ethereum::chainspec::ForkId {
        self.inner.latest_fork_id()
    }

    fn fork_filter(
        &self,
        head: reth_ethereum::chainspec::Head,
    ) -> reth_ethereum::chainspec::ForkFilter {
        self.inner.fork_filter(head)
    }
}

impl EthChainSpec for CustomChainSpec {
    type Header = Header;

    fn base_fee_params_at_block(
        &self,
        block_number: u64,
    ) -> reth_ethereum::chainspec::BaseFeeParams {
        self.inner.base_fee_params_at_block(block_number)
    }

    fn blob_params_at_timestamp(&self, timestamp: u64) -> Option<alloy_eips::eip7840::BlobParams> {
        self.inner.blob_params_at_timestamp(timestamp)
    }

    fn base_fee_params_at_timestamp(
        &self,
        timestamp: u64,
    ) -> reth_ethereum::chainspec::BaseFeeParams {
        self.inner.base_fee_params_at_timestamp(timestamp)
    }

    fn bootnodes(&self) -> Option<Vec<NodeRecord>> {
        self.inner.bootnodes()
    }

    fn chain(&self) -> reth_ethereum::chainspec::Chain {
        self.inner.chain()
    }

    fn deposit_contract(&self) -> Option<&reth_ethereum::chainspec::DepositContract> {
        self.inner.deposit_contract()
    }

    fn display_hardforks(&self) -> Box<dyn std::fmt::Display> {
        EthChainSpec::display_hardforks(&self.inner)
    }

    fn prune_delete_limit(&self) -> usize {
        self.inner.prune_delete_limit()
    }

    fn genesis(&self) -> &Genesis {
        self.inner.genesis()
    }

    fn genesis_hash(&self) -> B256 {
        self.inner.genesis_hash()
    }

    fn genesis_header(&self) -> &Self::Header {
        self.inner.genesis_header()
    }

    fn final_paris_total_difficulty(&self) -> Option<U256> {
        self.inner.get_final_paris_total_difficulty()
    }

    fn next_block_base_fee(&self, _parent: &Header, _target_timestamp: u64) -> Option<u64> {
        Some(0)
    }
}

impl EthereumHardforks for CustomChainSpec {
    fn ethereum_fork_activation(
        &self,
        fork: reth_ethereum::chainspec::EthereumHardfork,
    ) -> reth_ethereum::chainspec::ForkCondition {
        self.inner.ethereum_fork_activation(fork)
    }
}

impl EthExecutorSpec for CustomChainSpec {
    fn deposit_contract_address(&self) -> Option<Address> {
        self.inner.deposit_contract_address()
    }
}

/// Parser for [`CustomChainSpec`].
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct CustomChainSpecParser;

impl ChainSpecParser for CustomChainSpecParser {
    type ChainSpec = CustomChainSpec;
    const SUPPORTED_CHAINS: &'static [&'static str] =
        <EthereumChainSpecParser as ChainSpecParser>::SUPPORTED_CHAINS;

    fn parse(input: &str) -> eyre::Result<Arc<Self::ChainSpec>> {
        Ok(Arc::new(CustomChainSpec {
            inner: Arc::unwrap_or_clone(EthereumChainSpecParser::parse(input)?),
        }))
    }
}
