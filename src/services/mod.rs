// Services module for future microservice integration
// Currently, all data is served from the JSON mock data file

#[allow(dead_code)]
pub struct ConsensusService {
    pub service_url: Option<String>,
}

#[allow(dead_code)]
pub struct BlockchainService {
    pub service_url: Option<String>,
}

#[allow(dead_code)]
pub struct StorageService {
    pub service_url: Option<String>,
}

#[allow(dead_code)]
impl ConsensusService {
    pub fn new(service_url: Option<String>) -> Self {
        ConsensusService { service_url }
    }
}

#[allow(dead_code)]
impl BlockchainService {
    pub fn new(service_url: Option<String>) -> Self {
        BlockchainService { service_url }
    }
}

#[allow(dead_code)]
impl StorageService {
    pub fn new(service_url: Option<String>) -> Self {
        StorageService { service_url }
    }
}
