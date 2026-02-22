pub mod consensus_client;
pub mod server_client;

// Services 模块：用于未来与各微服务进行集成
// 当前阶段，大部分数据仍然来自本地 JSON 模拟数据文件

#[allow(dead_code)]
pub struct ConsensusService {
    pub service_url: Option<String>,
}
