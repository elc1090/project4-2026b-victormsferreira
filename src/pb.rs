use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthStoreRecord {
    pub collection_id: String,
    pub collection_name: String,
    pub id: String,
    pub email: String,
    pub email_visibility: bool,
    pub verified: bool,
    pub name: String,
}
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct AuthStore {
    pub token: String,
    pub record: AuthStoreRecord,
}

#[derive(Default)]
pub struct Pocketbase {
    addr: String,
    client: Client,
    pub auth_store: AuthStore,
}
#[derive(Default, Serialize, Deserialize)]
struct PasswordCred {
    identity: String,
    password: String,
}

impl Pocketbase {
    pub async fn get<T: for<'a> Deserialize<'a>>(&self, path: &str) -> Option<T> {
        let url: String = format!("{}/{}", self.addr, path);
        let response = self
            .client
            .get(url)
            .bearer_auth(&self.auth_store.token)
            .send()
            .await;
        if let Ok(body) = response {
            match body.json::<T>().await {
                Ok(result) => {
                    return Some(result);
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
        } else if let Err(e) = response {
            println!("{}", e);
        }
        None
    }

    pub async fn post<T: for<'a> Deserialize<'a>>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Option<T> {
        let url: String = format!("{}/{}", self.addr, path);
        let response = self
            .client
            .post(url)
            .bearer_auth(&self.auth_store.token)
            .json(body)
            .send()
            .await;

        if let Ok(body) = response {
            match body.json::<T>().await {
                Ok(result) => {
                    return Some(result);
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
        } else if let Err(e) = response {
            println!("{}", e);
        }
        None
    }

    pub async fn patch<T: for<'a> Deserialize<'a>>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Option<T> {
        let url: String = format!("{}/{}", self.addr, path);
        let response = self
            .client
            .patch(url)
            .json(body)
            .bearer_auth(&self.auth_store.token)
            .send()
            .await;
        if let Ok(body) = response {
            match body.json::<T>().await {
                Ok(result) => {
                    return Some(result);
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
        } else if let Err(e) = response {
            println!("{}", e);
        }
        None
    }
    pub async fn delete<T: for<'a> Deserialize<'a>>(&self, path: &str) -> Option<T> {
        let url: String = format!("{}/{}", self.addr, path);
        let response = self
            .client
            .delete(url)
            .bearer_auth(&self.auth_store.token)
            .send()
            .await;
        if let Ok(body) = response
            && let Ok(result) = body.json::<T>().await
        {
            return Some(result);
        }
        None
    }
}

impl Pocketbase {
    pub fn new(addr: &str) -> Pocketbase {
        Self {
            addr: addr.to_string(),
            client: Client::new(),
            auth_store: Default::default(),
        }
    }
}

impl Pocketbase {
    pub async fn auth_with_password(&mut self, collection: &str, identity: &str, password: &str) {
        let body = PasswordCred {
            identity: identity.to_string(),
            password: password.to_string(),
        };
        let path = format!("/api/collections/{}/auth-with-password", collection);
        let response = self.post::<AuthStore>(&path, &body).await;
        if let Some(new_auth) = response {
            self.auth_store = new_auth;
            println!("{}", &self.auth_store.token);
        } else {
            println!("Failed to authenticate with id and password");
        }
    }
}
