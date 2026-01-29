// Services module for future microservice integration
// Currently, all data is served from the JSON mock data file

pub struct ConsensusService {
    pub service_url: Option<String>,
}

pub struct BlockchainService {
    pub service_url: Option<String>,
}

pub struct StorageService {
    pub service_url: Option<String>,
}

impl ConsensusService {
    pub fn new(service_url: Option<String>) -> Self {
        ConsensusService { service_url }
    }
}

impl BlockchainService {
    pub fn new(service_url: Option<String>) -> Self {
        BlockchainService { service_url }
    }
}

impl StorageService {
    pub fn new(service_url: Option<String>) -> Self {
        StorageService { service_url }
    }
}
