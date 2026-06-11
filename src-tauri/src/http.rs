use tauri::{AppHandle, Manager};

pub const USER_AGENT: &str = "Aeropeks/0.1.0 (https://github.com/mortenlein/Aeropeks)";

/// Shared HTTP client managed in Tauri state: one connection pool for every
/// integration. No client-level timeout — callers set per-request timeouts.
#[derive(Clone)]
pub struct HttpClient(pub reqwest::Client);

impl HttpClient {
    pub fn new() -> Self {
        Self(
            reqwest::Client::builder()
                .user_agent(USER_AGENT)
                .build()
                .expect("static reqwest client config cannot fail"),
        )
    }
}

pub fn client(app: &AppHandle) -> reqwest::Client {
    app.state::<HttpClient>().0.clone()
}
