#[derive(Debug, Clone)]
pub struct Config {
    pub server_addr: String,
    pub server_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server_addr: "127.0.0.1".to_string(),
            server_port: 8080,
        }
    }
}
