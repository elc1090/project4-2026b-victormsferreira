use reqwest::Client;
use serde::Serialize;
#[derive(Default)]
struct AuthStore {
    pub id: String,
    pub collection: String,
    pub email: String,
    pub verified: bool,
    pub token: String,
}

#[derive(Default)]
pub struct Pocketbase {
    addr: String,
    client: Client,
    auth_store: AuthStore,
}

impl Pocketbase {
    pub async fn get(&self, path: &str) -> Option<String> {
        let url: String = format!("{}/{}", self.addr, path);
        let response = self.client.get(url).send().await;
        if let Ok(body) = response
            && let Ok(text) = body.text().await
        {
            return Some(text);
        }
        None
    }

    pub async fn post(&self, path: &str, body: &impl Serialize) -> Option<String> {
        let url: String = format!("{}/{}", self.addr, path);
        let response = self.client.post(url).json(body).send().await;
        if let Ok(body) = response
            && let Ok(text) = body.text().await
        {
            return Some(text);
        }
        None
    }

    pub async fn patch(&self, path: &str, body: &impl Serialize) -> Option<String> {
        let url: String = format!("{}/{}", self.addr, path);
        let response = self.client.patch(url).json(body).send().await;
        if let Ok(body) = response
            && let Ok(text) = body.text().await
        {
            return Some(text);
        }
        None
    }
    pub async fn delete(&self, path: &str) -> Option<String> {
        let url: String = format!("{}/{}", self.addr, path);
        let response = self.client.delete(url).send().await;
        if let Ok(body) = response
            && let Ok(text) = body.text().await
        {
            return Some(text);
        }
        None
    }

    pub fn new(addr: &str) -> Pocketbase {
        Self {
            addr: addr.to_string(),
            client: Client::new(),
            auth_store: Default::default(),
        }
    }
}
