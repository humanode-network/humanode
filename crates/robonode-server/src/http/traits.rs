//! The logic operation related trait.

/// The trait to make logiс operations.
#[async_trait::async_trait]
pub trait LogicOp<Request> {
    /// Logic operation Response type.
    type Response;
    /// Logic operation Error type.
    type Error;

    /// Process logic operation request.
    async fn call(&self, req: Request) -> Result<Self::Response, Self::Error>;
}
