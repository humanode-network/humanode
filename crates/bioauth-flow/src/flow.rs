//! The flow implementation.

use std::marker::PhantomData;

use robonode_client::{AuthenticateRequest, EnrollRequest};

#[async_trait::async_trait]
pub trait RpcControl {
    type Error;

    async fn obtain_facescan(&mut self) -> Result<Vec<u8>, Self::Error>;
}

/// Signer provides signatures for the data.
pub trait Signer {
    /// Sign the provided data and return the signature.
    fn sign<D: AsRef<[u8]>>(&self, data: &D) -> Vec<u8>;
}

/// The necessary components for the bioauth flow.
pub struct Flow<PK, RPC> {
    /// The RPC control mechanism, used to obtain the data into the flow from an RPC.
    pub rpc: RPC,
    /// The Robonode API client.
    pub robonode_client: robonode_client::Client,
    /// The type used to encode the public key.
    pub public_key_type: PhantomData<PK>,
}

impl<PK, RPC> Flow<PK, RPC>
where
    PK: AsRef<[u8]>,
    RPC: RpcControl,
    <RPC as RpcControl>::Error: Send + Sync + std::error::Error + 'static,
{
    /// The bioauth flow.
    pub async fn enroll(&mut self, public_key: PK) -> Result<(), anyhow::Error> {
        let face_scan = self.rpc.obtain_facescan().await?;

        self.robonode_client
            .enroll(EnrollRequest {
                face_scan: face_scan.as_ref(),
                public_key: public_key.as_ref(),
            })
            .await?;

        Ok(())
    }
}

impl<PK, RPC> Flow<PK, RPC>
where
    PK: Signer,
    RPC: RpcControl,
    <RPC as RpcControl>::Error: Send + Sync + std::error::Error + 'static,
{
    /// The bioauth flow.
    pub async fn authenticate(
        &mut self,
        public_key: PK,
    ) -> Result<robonode_client::AuthenticateResponse, anyhow::Error> {
        let face_scan = self.rpc.obtain_facescan().await?;

        let signature = public_key.sign(&face_scan);

        let response = self
            .robonode_client
            .authenticate(AuthenticateRequest {
                face_scan: face_scan.as_ref(),
                face_scan_signature: signature.as_ref(),
            })
            .await?;

        Ok(response)
    }
}
