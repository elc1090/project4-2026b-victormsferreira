use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use std::vec::Vec;
const STORAGE_KEY: &str = "auth-store";

#[derive(Debug)]
pub enum PbErr {
    GenericError,
    ParseFail,
    RequestFail,
}
#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Record<T> {
    pub collection_id: String,
    pub collection_name: String,
    pub id: String,
    pub created: String,
    pub updated: String,
    #[serde(flatten)]
    pub item: T,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthStoreRecord {
    pub collection_id: String,
    pub collection_name: String,
    pub id: String,
    pub email: String,
    pub email_visibility: bool,
    pub verified: bool,
    pub name: Option<String>,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct AuthStore {
    pub token: String,
    pub record: AuthStoreRecord,
}

#[derive(Default, PartialEq, Clone)]
pub struct Pocketbase {
    pub addr: String,
    //client: Client,
    pub auth_store: Option<AuthStore>,
}
#[derive(Default, Serialize, Deserialize)]
struct PasswordCred {
    identity: String,
    password: String,
}

impl AuthStore {
    fn clear() -> Self {
        Self {
            ..Default::default()
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuth2<T> {
    pub provider: String,
    pub code: String,
    pub code_verifier: String,
    #[allow(non_snake_case)]
    pub redirect_URL: String,
    pub create_data: Option<T>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethodMFA {
    pub enabled: bool,
    pub duration: i32,
}
#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethodOTP {
    pub enabled: bool,
    pub duration: i32,
}

#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethodPassword {
    pub enabled: bool,
    pub identity_fields: Vec<String>,
}
#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethodOAuth2Provider {
    pub state: String,
    pub auth_url: String,
    pub code_verifier: String,
    pub code_challenge: String,
    pub name: String,
    pub logo: String,
    pub code_challenge_method: String,
    pub display_name: String,
}
#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethodOAuth2 {
    pub enabled: bool,
    pub providers: Vec<AuthMethodOAuth2Provider>,
}
#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethods {
    pub mfa: AuthMethodMFA,
    pub otp: AuthMethodOTP,
    pub password: AuthMethodPassword,
    pub oauth2: AuthMethodOAuth2,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Records<T> {
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
    pub total_items: usize,
    pub items: Vec<Record<T>>,
}

// Implementation of basic API methods
impl Pocketbase {
    pub async fn get<T: for<'a> Deserialize<'a>>(
        &self,
        path: &str,
        params: Vec<(&str, &str)>,
    ) -> Result<T, PbErr> {
        let url: String = format!("{}/{}", self.addr, path);
        let request = Request::get(&url);
        let response = if let Some(auth_store) = &self.auth_store {
            request.header("Authorization", &auth_store.token)
        } else {
            request
        }
        .query(params)
        .send()
        .await;
        if let Ok(body) = response {
            match body.json::<T>().await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(_e) => {
                    return Err(PbErr::ParseFail);
                }
            }
        }

        Err(PbErr::RequestFail)
    }

    pub async fn post<T: for<'a> Deserialize<'a>>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T, PbErr> {
        let url: String = format!("{}/{}", self.addr, path);
        let request = Request::post(&url);
        let response = if let Some(auth_store) = &self.auth_store {
            request.header("Authorization", &auth_store.token)
        } else {
            request
        }
        .json(body)
        .unwrap()
        .send()
        .await;

        if let Ok(body) = response {
            match body.json::<T>().await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(_e) => {
                    return Err(PbErr::ParseFail);
                }
            }
        }
        Err(PbErr::RequestFail)
    }

    pub async fn patch<T: for<'a> Deserialize<'a>>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T, PbErr> {
        let url: String = format!("{}/{}", self.addr, path);
        let request = Request::patch(&url);
        let response = if let Some(auth_store) = &self.auth_store {
            request.header("Authorization", &auth_store.token)
        } else {
            request
        }
        .json(body)
        .unwrap()
        .send()
        .await;
        if let Ok(body) = response {
            match body.json::<T>().await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(_e) => {
                    return Err(PbErr::ParseFail);
                }
            }
        }
        Err(PbErr::RequestFail)
    }
    pub async fn delete(&self, path: &str) -> Result<(), PbErr> {
        let url: String = format!("{}/{}", self.addr, path);
        let request = Request::delete(&url);
        let response = if let Some(auth_store) = &self.auth_store {
            request.header("Authorization", &auth_store.token)
        } else {
            request
        }
        .send()
        .await;
        if response.is_ok() {
            return Ok(());
        }
        Err(PbErr::RequestFail)
    }
}

impl Pocketbase {
    pub fn new(addr: &str) -> Pocketbase {
        Self {
            addr: addr.to_string(),
            //client: Client::new(),
            auth_store: if let Ok(stored) = LocalStorage::get::<AuthStore>(STORAGE_KEY) {
                Some(stored)
            } else {
                None
            },
        }
    }
}

// Implementation of auth methods
impl Pocketbase {
    pub fn logout(&self) -> Self {
        let _ = LocalStorage::set::<Option<AuthStore>>(STORAGE_KEY, None);
        Self {
            auth_store: None,
            ..self.clone()
        }
    }
    pub async fn auth_with_password(
        &self,
        collection: &str,
        identity: &str,
        password: &str,
    ) -> Self {
        let body = PasswordCred {
            identity: identity.to_string(),
            password: password.to_string(),
        };
        let path = format!("api/collections/{}/auth-with-password", collection);
        let response = self.post::<AuthStore>(&path, &body).await;
        Self {
            auth_store: if let Ok(new_auth) = response {
                let _ = LocalStorage::set(STORAGE_KEY, &new_auth);
                Some(new_auth)
            } else {
                None
            },
            ..self.clone()
        }
    }
    pub async fn list_auth_methods(&self, collection: &str) -> AuthMethods {
        let path = format!("api/collections/{}/auth-methods", collection);
        let response = self.get::<AuthMethods>(&path, vec![]).await;
        if let Ok(auth_methods) = response {
            auth_methods
        } else {
            AuthMethods {
                ..Default::default()
            }
        }
    }
    pub async fn auth_with_oauth2<T: for<'a> Deserialize<'a> + Serialize>(
        &self,
        collection: &str,
        data: &OAuth2<T>,
    ) -> Self {
        let path = format!("api/collections/{}/auth-with-oauth2", collection);
        let response = self.post::<AuthStore>(&path, data).await;
        Self {
            auth_store: if let Ok(new_auth) = response {
                let _ = LocalStorage::set(STORAGE_KEY, &new_auth);
                Some(new_auth)
            } else {
                None
            },
            ..self.clone()
        }
    }
}

#[derive(Default, Clone)]
pub struct ListParams {
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub expand: Option<String>,
    pub fields: Option<String>,
}

impl ListParams {
    pub fn new() -> ListParams {
        Self {
            filter: None,
            sort: None,
            expand: None,
            fields: None,
        }
    }
    pub fn filter(&self, s: &str) -> Self {
        Self {
            filter: Some(s.to_string()),
            ..self.clone()
        }
    }
    pub fn sort(&mut self, s: &str) -> Self {
        Self {
            sort: Some(s.to_string()),
            ..self.clone()
        }
    }
    pub fn expand(&mut self, s: &str) -> Self {
        Self {
            expand: Some(s.to_string()),
            ..self.clone()
        }
    }
    pub fn fields(&mut self, s: &str) -> Self {
        Self {
            fields: Some(s.to_string()),
            ..self.clone()
        }
    }
}
// Implementation of CRUD methods
impl Pocketbase {
    pub async fn full_list<T: for<'a> Deserialize<'a>>(
        &self,
        collection: &str,
        params: Option<&ListParams>,
    ) -> Vec<Record<T>> {
        let mut result: Vec<Record<T>> = vec![];
        let batch_size: usize = 1000;
        let mut page: usize = 1;
        let mut done = false;
        while !done {
            let record = self
                .list(collection, page, batch_size, params)
                .await
                .unwrap();
            if record.items.len() < record.per_page {
                done = true;
            }
            result.extend(record.items);
            page += 1;
        }
        result
    }
    pub async fn get_first_list_item<T: for<'a> Deserialize<'a> + Clone>(
        &self,
        collection: &str,
        params: Option<&ListParams>,
    ) -> Option<Record<T>> {
        let list: Result<Records<T>, PbErr> = self.list(collection, 1, 1, params).await;
        if let Ok(l) = list
            && !l.items.is_empty()
        {
            return Some(l.items[0].clone());
        }
        None
    }
    pub async fn list<T: for<'a> Deserialize<'a>>(
        &self,
        collection: &str,
        page: usize,
        per_page: usize,
        params: Option<&ListParams>,
    ) -> Result<Records<T>, PbErr> {
        let page_str = page.to_string();
        let per_page_str = per_page.to_string();
        let mut p: Vec<(&str, &str)> = vec![("page", &page_str), ("perPage", &per_page_str)];
        if let Some(params) = params {
            if let Some(ref f) = params.filter {
                p.push(("filter", f));
            }
            if let Some(ref s) = params.sort {
                p.push(("sort", s));
            }
            if let Some(ref e) = params.expand {
                p.push(("expand", e));
            }
            if let Some(ref f) = params.fields {
                p.push(("fields", f));
            }
        }
        let path = format!("api/collections/{}/records", collection);
        self.get(&path, p).await
    }
    pub async fn view<T: for<'a> Deserialize<'a>>(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<Record<T>, PbErr> {
        let path = format!("api/collections/{}/records/{}", collection, id);
        self.get(&path, vec![]).await
    }
    pub async fn create<T: for<'a> Deserialize<'a> + Serialize>(
        &self,
        collection: &str,
        body: &T,
    ) -> Result<Record<T>, PbErr> {
        let path = format!("api/collections/{}/records", collection);
        self.post(&path, &body).await
    }
    pub async fn update<T: for<'a> Deserialize<'a> + Serialize>(
        &self,
        collection: &str,
        id: &str,
        body: &T,
    ) -> Result<Record<T>, PbErr> {
        let path = format!("api/collections/{}/records/{}", collection, id);
        self.patch(&path, &body).await
    }
    pub async fn remove(&self, collection: &str, id: &str) {
        let path = format!("api/collections/{}/records/{}", collection, id);
        let _ = self.delete(&path).await;
    }
}
