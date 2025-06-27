use eyre::Result;
use reth_ethereum::{
    chainspec::ChainSpec,
    evm::{
        primitives::{
            Database, EthEvm, EthEvmFactory, Evm, EvmFactory,
            eth::EthEvmContext,
            precompiles::{PrecompileInput, PrecompilesMap},
        },
        revm::{
            Inspector,
            context::{
                TxEnv,
                result::{EVMError, HaltReason},
            },
            precompile::secp256r1::{P256VERIFY, p256_verify},
            primitives::hardfork::SpecId,
        },
    },
    node::{
        EthEvmConfig, EthereumEngineValidatorBuilder, EthereumEthApiBuilder, EthereumNode,
        api::{FullNodeComponents, FullNodeTypes, NodeTypes},
        builder::{
            BuilderContext, DebugNode, Node, NodeAdapter, NodeComponentsBuilder,
            PayloadBuilderConfig,
            components::{BasicPayloadServiceBuilder, ComponentsBuilder, ExecutorBuilder},
        },
        node::{
            EthereumAddOns, EthereumConsensusBuilder, EthereumNetworkBuilder,
            EthereumPayloadBuilder, EthereumPoolBuilder,
        },
    },
};
use std::error::Error;

#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct CustomEvmFactory {
    inner: EthEvmFactory,
}

impl CustomEvmFactory {
    fn customize_evm<DB: Database, I: Inspector<EthEvmContext<DB>>>(
        &self,
        evm: &mut EthEvm<DB, I, PrecompilesMap>,
    ) {
        if evm.cfg.spec >= SpecId::PRAGUE {
            evm.precompiles_mut().apply_precompile(P256VERIFY.address(), |_| {
                let p = |input: PrecompileInput<'_>| p256_verify(input.data, input.gas);
                Some(p.into())
            });
        }
    }
}

impl EvmFactory for CustomEvmFactory {
    type Evm<DB: Database, I: Inspector<Self::Context<DB>>> = EthEvm<DB, I, PrecompilesMap>;
    type Context<DB: Database> = EthEvmContext<DB>;
    type Tx = TxEnv;
    type Error<DBError: Error + Send + Sync + 'static> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Spec = SpecId;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(
        &self,
        db: DB,
        evm_env: reth_ethereum::evm::primitives::EvmEnv<Self::Spec>,
    ) -> Self::Evm<DB, reth_ethereum::evm::revm::inspector::NoOpInspector> {
        let mut evm = self.inner.create_evm(db, evm_env);

        self.customize_evm(&mut evm);

        evm
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>>>(
        &self,
        db: DB,
        input: reth_ethereum::evm::primitives::EvmEnv<Self::Spec>,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        let mut evm = self.inner.create_evm_with_inspector(db, input, inspector);

        self.customize_evm(&mut evm);

        evm
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct CustomExecutorBuilder;

impl<N: FullNodeTypes<Types = CustomNode>> ExecutorBuilder<N> for CustomExecutorBuilder {
    type EVM = EthEvmConfig<ChainSpec, CustomEvmFactory>;

    async fn build_evm(self, ctx: &BuilderContext<N>) -> Result<Self::EVM> {
        Ok(EthEvmConfig::new_with_evm_factory(
            ctx.chain_spec().clone(),
            CustomEvmFactory::default(),
        )
        .with_extra_data(ctx.payload_builder_config().extra_data_bytes()))
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct CustomNode;

impl NodeTypes for CustomNode {
    type Primitives = <EthereumNode as NodeTypes>::Primitives;
    type ChainSpec = <EthereumNode as NodeTypes>::ChainSpec;
    type StateCommitment = <EthereumNode as NodeTypes>::StateCommitment;
    type Storage = <EthereumNode as NodeTypes>::Storage;
    type Payload = <EthereumNode as NodeTypes>::Payload;
}

impl<N: FullNodeTypes<Types = Self>> Node<N> for CustomNode {
    type ComponentsBuilder = ComponentsBuilder<
        N,
        EthereumPoolBuilder,
        BasicPayloadServiceBuilder<EthereumPayloadBuilder>,
        EthereumNetworkBuilder,
        CustomExecutorBuilder,
        EthereumConsensusBuilder,
    >;

    type AddOns = EthereumAddOns<
        NodeAdapter<N, <Self::ComponentsBuilder as NodeComponentsBuilder<N>>::Components>,
        EthereumEthApiBuilder,
        EthereumEngineValidatorBuilder<ChainSpec>,
    >;

    fn components_builder(&self) -> Self::ComponentsBuilder {
        EthereumNode::components().executor(CustomExecutorBuilder::default())
    }

    fn add_ons(&self) -> Self::AddOns {
        EthereumAddOns::default()
    }
}

impl<N: FullNodeComponents<Types = Self>> DebugNode<N> for CustomNode {
    type RpcBlock = reth_ethereum::rpc::eth::primitives::Block;

    fn rpc_to_primitive_block(
        rpc_block: Self::RpcBlock,
    ) -> reth_ethereum::node::api::BlockTy<Self> {
        rpc_block.into_consensus().convert_transactions()
    }
}
