pub const CONNECTION_CONFIG: &str = include_str!("connection-config.json");

pub fn connection_config() -> String {
    CONNECTION_CONFIG.to_string()
}
