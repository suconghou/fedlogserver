use mongodb::bson::{to_bson, Document};
use mongodb::{options::ClientOptions, Client};
use serde_json::Value;
use std::env;

pub struct StoreTask {
    client: Option<Client>,
    database: String,
}

impl StoreTask {
    pub async fn new() -> Self {
        let db = env::var("MONGODB_DATABASE");
        let uri = env::var("MONGODB_URI");
        if db.is_err() || uri.is_err() {
            return StoreTask {
                client: None,
                database: "".to_owned(),
            };
        }
        let mut client_options = ClientOptions::parse(uri.unwrap()).await.unwrap();
        client_options.app_name = Some("ws".to_string());
        return StoreTask {
            client: Some(Client::with_options(client_options).unwrap()),
            database: db.unwrap(),
        };
    }

    pub fn ok(&self) -> bool {
        self.client.is_some()
    }

    pub async fn save(&self, collection: &str, data: Value) {
        if self.client.is_none() {
            return;
        }
        let res = to_bson(&data);
        if res.is_err() {
            return;
        }
        let db = self.client.as_ref().unwrap().database(&self.database);
        let collection = db.collection::<Document>(collection);
        let r = collection
            .insert_one(res.unwrap().as_document().unwrap(), None)
            .await;
        if r.is_err() {
            println!("{:?}", r.err());
        }
    }
}