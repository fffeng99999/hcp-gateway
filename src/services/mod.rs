pub mod consensus_service;
pub mod blockchain_service;
pub mod storage_service;

/// Service integrator trait for standardized service interaction
pub trait ServiceIntegrator: Send + Sync {
    /// Service health check
    async fn health_check(&self) -> Result<bool, String>;
}
