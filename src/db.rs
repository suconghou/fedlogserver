use mongodb::bson::{to_bson, DateTime, Document};
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
        let client_options = ClientOptions::parse(uri.unwrap()).await.unwrap_or_default();
        return StoreTask {
            client: Client::with_options(client_options).ok(),
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
        match to_bson(&data) {
            Ok(mut b) => match b.as_document_mut() {
                Some(doc) => {
                    doc.insert("createdAt", DateTime::now());
                    let db = self.client.as_ref().unwrap().database(&self.database);
                    let collection = db.collection::<Document>(collection);
                    let r = collection.insert_one(doc, None).await;
                    if r.is_err() {
                        println!("{:?}", r.err());
                    }
                }
                None => (),
            },
            Err(e) => println!("{:?}", e),
        }
    }
}
