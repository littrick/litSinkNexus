use std::sync::Mutex;


#[derive(Debug)]
pub struct AppConfig {
    pub auto_connect: Mutex<bool>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_connect: Mutex::new(true),
        }
    }
}
