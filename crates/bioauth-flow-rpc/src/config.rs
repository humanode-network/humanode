use crate::{signer, Signer};

pub trait Config: Send + Sync + 'static {
    type RobonodeClient: AsRef<robonode_client::Client> + Send + Sync;

    type ValidatorPublicKeyType: sp_api::Encode + AsRef<[u8]> + Clone + Send + Sync;

    type ValidatorKeyExtractor: bioauth_keys::traits::KeyExtractor<
            PublicKeyType = Self::ValidatorPublicKeyType,
            Error: std::fmt::Debug,
        > + Send
        + Sync;
    type ValidatorSignerFactory: signer::Factory<
            Vec<u8>,
            <Self::ValidatorKeyExtractor as bioauth_keys::traits::KeyExtractor>::PublicKeyType,
            Signer: Signer<Vec<u8>, Error: std::error::Error + 'static> + Send + Sync,
        > + Send
        + Sync;

    type Block: sp_api::BlockT;
    type Client: sp_blockchain::HeaderBackend<Self::Block>
        + sp_api::ProvideRuntimeApi<
            Self::Block,
            Api: bioauth_flow_api::BioauthFlowApi<
                Self::Block,
                <Self::ValidatorKeyExtractor as bioauth_keys::traits::KeyExtractor>::PublicKeyType,
                Self::Timestamp,
            >,
        > + Send
        + Sync
        + 'static;

    type Timestamp: sp_api::Decode;

    type TransactionPool: sc_transaction_pool_api::TransactionPool<Block = Self::Block>;
}

pub struct Generic<
    RobonodeClient,
    ValidatorPublicKeyType,
    ValidatorKeyExtractor,
    ValidatorSignerFactory,
    Block,
    Client,
    Timestamp,
    TransactionPool,
>(
    std::convert::Infallible,
    std::marker::PhantomData<(
        RobonodeClient,
        ValidatorPublicKeyType,
        ValidatorKeyExtractor,
        ValidatorSignerFactory,
        Block,
        Client,
        Timestamp,
        TransactionPool,
    )>,
);

impl<
        RobonodeClient,
        ValidatorPublicKeyType,
        ValidatorKeyExtractor,
        ValidatorSignerFactory,
        Block,
        Client,
        Timestamp,
        TransactionPool,
    > Config
    for Generic<
        RobonodeClient,
        ValidatorPublicKeyType,
        ValidatorKeyExtractor,
        ValidatorSignerFactory,
        Block,
        Client,
        Timestamp,
        TransactionPool,
    >
where
    ValidatorKeyExtractor: bioauth_keys::traits::KeyExtractor<
            PublicKeyType = Self::ValidatorPublicKeyType,
            Error: std::fmt::Debug,
        > + Send
        + Sync,
    ValidatorSignerFactory: signer::Factory<
            Vec<u8>,
            <Self::ValidatorKeyExtractor as bioauth_keys::traits::KeyExtractor>::PublicKeyType,
            Signer: Signer<Vec<u8>, Error: std::error::Error + 'static> + Send + Sync,
        > + Send
        + Sync,

    Block: sp_api::BlockT,
    Client: sp_blockchain::HeaderBackend<Self::Block>
        + sp_api::ProvideRuntimeApi<
            Self::Block,
            Api: bioauth_flow_api::BioauthFlowApi<
                Self::Block,
                <Self::ValidatorKeyExtractor as bioauth_keys::traits::KeyExtractor>::PublicKeyType,
                Self::Timestamp,
            >,
        > + Send
        + Sync
        + 'static,

    Timestamp: sp_api::Decode,

    TransactionPool: sc_transaction_pool_api::TransactionPool<Block = Self::Block>,
{
    type RobonodeClient = RobonodeClient;
    type ValidatorPublicKeyType = ValidatorPublicKeyType;
    type ValidatorKeyExtractor = ValidatorKeyExtractor;
    type ValidatorSignerFactory = ValidatorSignerFactory;
    type Block = Block;
    type Client = Client;
    type Timestamp = Timestamp;
    type TransactionPool = TransactionPool;
}
