use futures::TryStreamExt;
use mongodb::bson::{doc, to_bson, DateTime, Document};
use mongodb::Database;
use mongodb::{options::ClientOptions, Client};
use serde_json::Value;
use std::env;
use std::time::{Duration, SystemTime};

pub struct DbConnection {
    db: Option<Database>,
}

impl DbConnection {
    pub async fn new() -> Self {
        let Ok(db) = env::var("MONGODB_DATABASE") else {
            return Self { db: None };
        };
        let Ok(uri) = env::var("MONGODB_URI") else {
            return Self { db: None };
        };
        let client_options = ClientOptions::parse(uri).await.unwrap_or_default();
        match Client::with_options(client_options) {
            Ok(client) => Self {
                db: Some(client.database(&db)),
            },
            Err(_) => Self { db: None },
        }
    }

    pub fn ok(&self) -> bool {
        self.db.is_some()
    }

    pub async fn save(&self, collection: &str, data: Value) {
        let Some(db) = &self.db else {
            return;
        };
        match to_bson(&data) {
            Ok(mut b) => match b.as_document_mut() {
                Some(doc) => {
                    doc.insert("createdAt", DateTime::now());
                    let collection = db.collection::<Document>(collection);
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
        let Some(db) = &self.db else {
            return Ok(result);
        };
        let pipeline = build_query(params);
        let options = None;
        let collection = db.collection::<Document>(collection);
        let mut cursor = collection.aggregate(pipeline, options).await?;
        while let Some(item) = cursor.try_next().await? {
            result.push(item);
        }
        Ok(result)
    }
}

fn build_query(params: Document) -> Vec<Document> {
    let mut pipeline = Vec::new();
    let mut _match = doc! {"createdAt": build_time_filter(&params)};
    let mut _group = doc! {};
    let mut _project = doc! {};
    let mut _count = doc! {};
    let mut _sort = doc! {
        "$sort":{
            "createdAt":-1,
        },
    };
    let mut _limit = doc! {
        "$limit":100
    };
    let mut _skip = doc! {};

    for (k, v) in params {
        let Some(val) = v.as_str() else {
            continue; // 这个基本不会用到，因为我们是从http query上解析的Document，键值都是string类型
        };
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
                if !key.starts_with("$") {
                    _match.insert(
                        key,
                        doc! {
                            "$exists":false,
                        },
                    );
                }
            }
            "$addToSet" => {
                if key.starts_with("$") {
                    _group.insert(
                        key.trim_start_matches("$"),
                        doc! {
                            "$addToSet":key,
                        },
                    );
                } else {
                    _group.insert(
                        key,
                        doc! {
                            "$addToSet":"$".to_owned()+key,
                        },
                    );
                }
            }
            "$project" => {
                _project.insert("_id", 0);
                _project.insert(key.trim_start_matches("$"), 1);
            }

            _ => match key {
                "$count" => {
                    _count.insert("$count", val.trim_start_matches("$"));
                }
                "$group" => {
                    if val.contains("%") {
                        _group.insert(
                            "_id",
                            doc! { "$dateToString":{"format":val, "date":"$createdAt","timezone":"Asia/Shanghai"} },
                        );
                    } else {
                        _group.insert("_id", val);
                    }
                    _group.insert("count", doc! {"$sum":1});
                    _sort.insert("$sort", doc! {"count":-1});
                }
                "$sort" => {
                    if val.starts_with("$") {
                        _sort.insert("$sort", doc! {val.trim_start_matches("$"):1});
                    } else {
                        _sort.insert("$sort", doc! {val:-1});
                    }
                }
                "$limit" => {
                    if let Ok(num) = val.parse::<i64>() {
                        _limit.insert(
                            "$limit",
                            match num {
                                1..=1000 => num,
                                _ => 10,
                            },
                        );
                    }
                }
                "$skip" => {
                    if let Ok(num) = val.parse::<i64>() {
                        _skip.insert(
                            "$skip",
                            match num {
                                1..=1000 => num,
                                _ => 0,
                            },
                        );
                    }
                }
                _ => {
                    if !key.starts_with("$") {
                        _match.insert(key, val);
                    }
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
        if _group.contains_key("_id") {
            pipeline.push(doc! {"$group":_group})
        } else {
            pipeline.push(doc! {"$project":_project});
        }
        pipeline.push(_sort);
        pipeline.push(_limit);
        if !_skip.is_empty() {
            pipeline.push(_skip)
        }
    } else {
        pipeline.push(_count);
    }

    pipeline
}

fn build_time_filter(params: &Document) -> Document {
    let mut created_at = doc! {};
    if let Ok(v) = params.get_str("$lt") {
        if let Ok(n) = v.parse::<i64>() {
            created_at.insert("$lt", DateTime::from_millis(1000 * n));
        }
    }
    if let Ok(v) = params.get_str("$gt") {
        if let Ok(n) = v.parse::<i64>() {
            created_at.insert("$gt", DateTime::from_millis(1000 * n));
            return created_at;
        }
    }
    created_at.insert(
        "$gt",
        DateTime::from_system_time(SystemTime::now() - Duration::new(3600, 0)),
    );
    created_at
}
