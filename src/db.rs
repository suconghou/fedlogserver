use futures::TryStreamExt;
use mongodb::bson::{doc, to_bson, DateTime, Document};
use mongodb::Database;
use mongodb::{options::ClientOptions, Client};
use serde_json::Value;
use std::env;

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
                        println!("{:?}", r.err().unwrap());
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
        params: Document,
    ) -> Result<Vec<Document>, Box<dyn std::error::Error>> {
        let mut result: Vec<Document> = Vec::new();
        let pipeline = build_query(params);
        if self.db.is_none() {
            return Ok(result);
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

fn build_query(params: Document) -> Vec<Document> {
    let mut pipeline = Vec::new();
    let hour = params.get_str("$gt").unwrap_or("6").parse().unwrap_or(6);
    let gt = recent(hour);
    let mut _match = doc! {
        "createdAt":{
            "$gt":DateTime::from_system_time(gt),
        }
    };
    let mut _group = doc! {};
    let mut _project = doc! {};
    let mut _count = doc! {};
    let mut _sort_limit_skip = doc! {
        "$sort":{
            "createdAt":-1,
        }
    };
    for (k, v) in params {
        if v.as_str().is_none() {
            continue; // 这个基本不会用到，因为我们是从http query上解析的Document，键值都是string类型
        }
        let val = v.as_str().unwrap();
        let key = k.as_str();
        match val {
            "$exists" => {
                _match.insert(
                    key,
                    doc! {
                        "$exists":true,
                    },
                );
            }
            "" => {
                _match.insert(
                    key,
                    doc! {
                        "$exists":false,
                    },
                );
            }
            "$addToSet" => {
                _group.insert(
                    key,
                    doc! {
                        "$addToSet":key,
                    },
                );
            }
            "$project" => {
                _project.insert("_id", 0);
                _project.insert(key, 1);
            }

            _ => match key {
                "$gt" => {}
                "$lg" => {}
                "$count" => {
                    _count.insert("$count", val);
                }
                "$group" => {
                    _group.insert("_id", val);
                    _group.insert("count", doc! {"$sum":1});
                    _sort_limit_skip.insert("$sort", doc! {"count":-1});
                }
                "$sort" => {
                    _sort_limit_skip.insert("$sort", doc! {val:-1});
                }
                "$limit" => {
                    _sort_limit_skip.insert("$limit", val);
                }
                "$skip" => {
                    _sort_limit_skip.insert("$skip", val);
                }
                _ => {
                    _match.insert(key, val);
                }
            },
        };
    }
    if _project.is_empty() {
        _project = doc! {
            "_id":0,
            "createdAt":0,
        };
    }
    pipeline.push(doc! {"$match":_match});
    if _count.is_empty() {
        if _group.is_empty() {
            pipeline.push(doc! {"$project":_project});
        } else {
            pipeline.push(doc! {"$group":_group})
        }
        pipeline.push(_sort_limit_skip);
    } else {
        pipeline.push(_count);
    }

    pipeline
}
