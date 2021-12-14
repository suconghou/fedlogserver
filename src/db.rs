use futures::TryStreamExt;
use mongodb::bson::{doc, to_bson, DateTime, Document};
use mongodb::Database;
use mongodb::{options::ClientOptions, Client};
use serde_json::Value;
use std::env;
use std::sync::Arc;

use crate::route::QueryOptions;
use crate::util::recent;

pub struct DbConnection {
    db: Option<Database>,
}

impl DbConnection {
    pub async fn new() -> Self {
        let db = env::var("MONGODB_DATABASE");
        let uri = env::var("MONGODB_URI");
        if db.is_err() || uri.is_err() {
            return Self { db: None };
        }
        let client_options = ClientOptions::parse(uri.unwrap()).await.unwrap_or_default();
        match Client::with_options(client_options) {
            Ok(client) => Self {
                db: Some(client.database(&db.unwrap())),
            },
            Err(_) => Self { db: None },
        }
    }

    pub fn ok(&self) -> bool {
        self.db.is_some()
    }

    pub async fn save(&self, collection: &str, data: Value) {
        if self.db.is_none() {
            return;
        }
        match to_bson(&data) {
            Ok(mut b) => match b.as_document_mut() {
                Some(doc) => {
                    doc.insert("createdAt", DateTime::now());
                    let collection = self.db.as_ref().unwrap().collection::<Document>(collection);
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

    pub async fn aggregate(
        &self,
        collection: &str,
        params: Arc<QueryOptions>,
    ) -> Result<Vec<Document>, Box<dyn std::error::Error>> {
        let mut result: Vec<Document> = Vec::new();
        if self.db.is_none() {
            return Ok(result);
        }
        let mut pipeline = Vec::new();
        pipeline.push(build_match_query(params.clone()));
        if params.count.is_some() {
            pipeline.push(doc! {
                "$count":params.count.as_ref().unwrap(),
            });
        } else {
            // group & sort & skip & limit
            if params.group.is_some() {
                pipeline.push(doc! {
                    "$group":{
                        "_id":params.group.as_ref().unwrap(),
                        "count":{"$sum":1},
                    }
                });
                pipeline.push(doc! {
                    "$sort":{
                        "count":-1,
                    }
                });
            } else {
                pipeline.push(doc! {
                    "$sort":{
                        "createdAt":-1,
                    }
                });
            }
            if params.skip.is_some() {
                pipeline.push(doc! {
                    "$skip":params.skip.unwrap(),
                });
            }
            pipeline.push(doc! {
                "$limit": match params.limit.unwrap_or(100) {
                    0 => 1,
                    100.. => 100,
                    n=>n,
                } ,
            });
        }
        let options = None;
        let collection = self.db.as_ref().unwrap().collection::<Document>(collection);
        let mut cursor = collection.aggregate(pipeline, options).await?;
        while let Some(item) = cursor.try_next().await? {
            result.push(item);
        }
        Ok(result)
    }
}

fn build_match_query(params: Arc<QueryOptions>) -> Document {
    let gt = recent(params.hours.unwrap_or(6));
    let mut _match = doc! {
        "createdAt":{
            "$gt":DateTime::from_system_time(gt),
        }
    };
    if params.r#type.is_some() {
        _match.insert("type", params.r#type.as_ref().unwrap());
    }
    if params.ip.is_some() {
        _match.insert("ip", params.ip.as_ref().unwrap());
    }
    if params.cookie.is_some() {
        _match.insert("cookie", params.cookie.as_ref().unwrap());
    }
    if params.ua.is_some() {
        _match.insert("ua", params.ua.as_ref().unwrap());
    }
    if params.href.is_some() {
        _match.insert("href", params.href.as_ref().unwrap());
    }
    _match
}
