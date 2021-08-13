pub struct ClientConfig {
    pub hostname: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub topic: String,
    pub subscription_id: String,
}

impl ClientConfig {
    pub fn new(hostname: &str, port: u16, user: &str, password: &str, topic: &str, subscription_id: &str) -> Self {
        ClientConfig {
            hostname: hostname.to_string(),
            password: password.to_string(),
            port,
            user: user.to_string(),
            subscription_id: subscription_id.to_string(),
            topic: topic.to_string()
        }
    }
}