pub mod consensus_client;
pub mod server_client;

// Services module for future microservice integration
// Currently, all data is served from the JSON mock data file

#[allow(dead_code)]
pub struct ConsensusService {
    pub service_url: Option<String>,
}
