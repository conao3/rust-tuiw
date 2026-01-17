pub const DEFAULT_PORT: u16 = 50051;
pub const DEFAULT_HOST: &str = "127.0.0.1";

pub fn get_daemon_endpoint() -> String {
    format!("http://{}:{}", DEFAULT_HOST, DEFAULT_PORT)
}
