use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[derive(Debug)]
pub enum PbErr {
    GenericError,
    ParseFail,
    RequestFail,
}

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Records<T> {
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
    pub total_items: usize,
    pub items: Vec<T>,
}

impl Pocketbase {
    pub async fn get<T: for<'a> Deserialize<'a>>(
        &self,
        path: &str,
        params: &(impl Serialize + ?Sized),
    ) -> Result<T, PbErr> {
        let url: String = format!("{}/{}", self.addr, path);
        let response = self
            .client
            .get(url)
            .query(params)
            .bearer_auth(&self.auth_store.token)
            .send()
            .await;
        if let Ok(body) = response {
            match body.json::<T>().await {
                Ok(result) => {
                    return Ok(result);
                }
                Err(e) => {
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
                    return Ok(result);
                }
                Err(e) => {
                    return Err(PbErr::ParseFail);
                }
            }
        } else if let Err(e) = response {
            println!("{}", e);
        }
        Err(PbErr::RequestFail)
    }

    pub async fn patch<T: for<'a> Deserialize<'a>>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T, PbErr> {
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
                    return Ok(result);
                }
                Err(e) => {
                    return Err(PbErr::ParseFail);
                }
            }
        } else if let Err(e) = response {
            println!("{}", e);
        }
        Err(PbErr::RequestFail)
    }
    pub async fn delete<T: for<'a> Deserialize<'a>>(&self, path: &str) -> Result<T, PbErr> {
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
            return Ok(result);
        }
        Err(PbErr::RequestFail)
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
    pub async fn auth_with_password(
        &mut self,
        collection: &str,
        identity: &str,
        password: &str,
    ) -> Option<String> {
        let body = PasswordCred {
            identity: identity.to_string(),
            password: password.to_string(),
        };
        let path = format!("/api/collections/{}/auth-with-password", collection);
        let response = self.post::<AuthStore>(&path, &body).await;
        if let Ok(new_auth) = response {
            self.auth_store = new_auth;
            Some(self.auth_store.record.id.clone())
        } else {
            None
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
    pub fn new(
        filter: Option<&str>,
        sort: Option<&str>,
        expand: Option<&str>,
        fields: Option<&str>,
    ) -> ListParams {
        Self {
            filter: filter.map(str::to_string),
            sort: sort.map(str::to_string),
            expand: expand.map(str::to_string),
            fields: fields.map(str::to_string),
        }
    }
}

impl Pocketbase {
    pub async fn full_list<T: for<'a> Deserialize<'a>>(
        &self,
        collection: &str,
        params: Option<ListParams>,
    ) -> Vec<T> {
        let mut result: Vec<T> = vec![];
        let batch_size: usize = 1000;
        let mut page: usize = 1;
        let mut done = false;
        while !done {
            let record = self
                .list(collection, page, batch_size, params.clone())
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
        params: Option<ListParams>,
    ) -> Option<T> {
        let list: Result<Records<T>, PbErr> = self.list(collection, 1, 1, params).await;
        if let Ok(l) = list
            && l.items.len() > 0
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
        params: Option<ListParams>,
    ) -> Result<Records<T>, PbErr> {
        let mut p: Vec<(&str, String)> = vec![
            ("page", page.to_string()),
            ("perPage", per_page.to_string()),
        ];
        if let Some(params) = params {
            if let Some(f) = params.filter {
                p.push(("filter", f));
            }
            if let Some(s) = params.sort {
                p.push(("sort", s));
            }
            if let Some(e) = params.expand {
                p.push(("expand", e));
            }
            if let Some(f) = params.fields {
                p.push(("fields", f));
            }
        }
        let path = format!("/api/collections/{}/records", collection);
        self.get(&path, &p).await
    }
}
