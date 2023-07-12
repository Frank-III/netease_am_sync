use std::sync::Arc;
use std::{thread, time::Duration};
use tokio::sync::RwLock;
// TODO: watch the great build video about how login qr check could update the cookies
#[derive(Debug)]
struct NeteaseApi {
    client: Arc<RwLock<reqwest::Client>>,
    base_uri: String,
    cookies: Option<HashMap<String, String>>,
}

impl NeteaseApi {
    async fn gen_qr_code(&self) {
        todo!()
    }

    async fn login_qr_check(&mut self) {
        todo!()
    }

    async fn user_account(&self) {
        todo!()
    }
}
