#[derive(Debug)]
pub struct Config {
    pub debug: bool,
    pub channel: String,
    pub username: String,
    pub token: String
}

impl Config {
    pub fn new(debug: bool, channel: &str, username: &str, token: &str) -> Self {
        Self {
            debug,
            channel: channel.to_string(),
            username: username.to_string(),
            token: token.to_string()
        }
    }
}