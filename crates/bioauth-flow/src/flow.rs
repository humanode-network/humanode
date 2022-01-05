//! The flow implementation.

use std::{marker::PhantomData, ops::Deref, sync::Arc};

use primitives_liveness_data::{LivenessData, OpaqueLivenessData};
use robonode_client::{AuthenticateRequest, EnrollRequest};

/// Something that can provide us with the [`LivenessData`].
/// Typically this would be implemented by a state-machine that powers the LDP, and interacts with
/// the handheld device to establish a session and capture the FaceScan and the rest of
/// the parameters that contribute to the [`LivenessData`].
#[async_trait::async_trait]
pub trait LivenessDataProvider {
    /// The error type that can occur while we're obtaining the [`LivenessData`].
    type Error;

    /// Obtain and return the [`LivenessData`].
    ///
    /// Takes `self` by `&mut` to allow internal state-machine to progress.
    async fn provide(&mut self) -> Result<LivenessData, Self::Error>;
}

/// Signer provides signatures for the data.
#[async_trait::async_trait]
pub trait Signer<S> {
    /// Signature error.
    /// Error may originate from communicating with HSM, or from a thread pool failure, etc.
    type Error;

    /// Sign the provided data and return the signature, or an error if the siging fails.
    async fn sign<'a, D>(&self, data: D) -> Result<S, Self::Error>
    where
        D: AsRef<[u8]> + Send + 'a;
}

/// The necessary components for the bioauth flow.
///
/// The goal of this component is to encapsulate interoperation with the handheld device
/// and the robonode.
pub struct Flow<LDP, RC, VPK, VS> {
    /// The provider of the liveness data.
    pub liveness_data_provider: LDP,
    /// The Robonode API client.
    pub robonode_client: RC,
    /// The type used to encode the public key.
    pub validator_public_key_type: PhantomData<VPK>,
    /// The type that provides signing with the validator private key.
    pub validator_signer_type: PhantomData<VS>,
}

impl<LDP, RC, VPK, VS> Flow<LDP, RC, VPK, VS>
where
    LDP: LivenessDataProvider,
{
    /// The common logic to obtain the plain [`LivenessData`] from a provider and  convert it to
    /// an [`OpaqueLivenessData`].
    async fn obtain_opaque_liveness_data(
        &mut self,
    ) -> Result<OpaqueLivenessData, <LDP as LivenessDataProvider>::Error> {
        let liveness_data = self.liveness_data_provider.provide().await?;
        Ok(OpaqueLivenessData::from(&liveness_data))
    }
}

impl<LDP, RC, VPK, VS> Flow<LDP, RC, VPK, VS>
where
    VS: Signer<Vec<u8>>,
    <VS as Signer<Vec<u8>>>::Error: Send + Sync + std::error::Error + 'static,
    LDP: LivenessDataProvider,
    <LDP as LivenessDataProvider>::Error: Send + Sync + std::error::Error + 'static,
    RC: Deref<Target = robonode_client::Client>,
    VPK: AsRef<[u8]>,
{
    /// The enroll flow.
    pub async fn enroll(&mut self, public_key: &VPK, signer: &VS) -> Result<(), anyhow::Error> {
        let opaque_liveness_data = self.obtain_opaque_liveness_data().await?;

        let signature = signer.sign(&opaque_liveness_data).await?;

        self.robonode_client
            .enroll(EnrollRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                public_key: public_key.as_ref(),
                liveness_data_signature: signature.as_ref(),
            })
            .await?;

        Ok(())
    }
}

impl<LDP, RC, VPK, VS> Flow<LDP, RC, VPK, VS>
where
    VS: Signer<Vec<u8>>,
    <VS as Signer<Vec<u8>>>::Error: Send + Sync + std::error::Error + 'static,
    LDP: LivenessDataProvider,
    <LDP as LivenessDataProvider>::Error: Send + Sync + std::error::Error + 'static,
    RC: Deref<Target = robonode_client::Client>,
{
    /// The authentication flow.
    ///
    /// Returns the authentication response, providing the auth ticket and its signature.
    pub async fn authenticate(
        &mut self,
        signer: &VS,
    ) -> Result<robonode_client::AuthenticateResponse, anyhow::Error> {
        let opaque_liveness_data = self.obtain_opaque_liveness_data().await?;

        let signature = signer.sign(&opaque_liveness_data).await?;

        let response = self
            .robonode_client
            .authenticate(AuthenticateRequest {
                liveness_data: opaque_liveness_data.as_ref(),
                liveness_data_signature: signature.as_ref(),
            })
            .await?;

        Ok(response)
    }
}
