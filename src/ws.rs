use crate::db::DbConnection;
use crate::queue::Queue;
use crate::util;
use actix::{Actor, Addr, AsyncContext, Handler, StreamHandler};
use actix_web::web::Bytes;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use serde_json::Value;
use std::time::Duration;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

lazy_static! {
    pub static ref USERS: Conns = Conns::new();
    pub static ref QUEUE: RwLock<Queue<QueueItem>> = RwLock::new(Queue::new());
}

pub struct Conns {
    data: RwLock<HashMap<u64, Addr<WsConn>>>,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct GroupMsg {
    pub group: String,
    pub data: String,
    pub bytes: Bytes,
}

pub struct QueueItem {
    pub ua: String,
    pub ip: String,
    pub refer: String,
    pub data: Arc<GroupMsg>,
}

impl Conns {
    fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
    fn insert(&self, id: u64, conn: Addr<WsConn>) {
        let mut u = self.data.write().unwrap();
        u.insert(id, conn);
    }

    fn remove(&self, id: u64) {
        let mut u = self.data.write().unwrap();
        u.remove(&id);
    }

    fn each(&self, msg: Arc<GroupMsg>) {
        let u = self.data.read().unwrap();
        for item in u.values() {
            item.do_send(msg.clone());
        }
    }

    pub fn stat(&self) -> Vec<u64> {
        let u = self.data.read().unwrap();
        let mut s: Vec<u64> = vec![];
        for key in u.keys() {
            s.push(*key);
        }
        return s;
    }
}

pub struct WsConn {
    pub id: u64,
    pub group: String,
}

impl Actor for WsConn {
    type Context = WebsocketContext<Self>;
    /// 连接上
    fn started(&mut self, ctx: &mut Self::Context) {
        USERS.insert(self.id, ctx.address());
    }

    /// 断开连接
    fn stopped(&mut self, _: &mut Self::Context) {
        USERS.remove(self.id);
    }
}

/// Handler for Message message
impl StreamHandler<Result<Message, ProtocolError>> for WsConn {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(Message::Text(text)) => USERS.each(Arc::new(GroupMsg {
                group: self.group.clone(),
                data: text.trim().to_string(),
                bytes: Bytes::new(),
            })),
            Ok(Message::Ping(msg)) => ctx.pong(&msg),
            Ok(Message::Binary(bin)) => USERS.each(Arc::new(GroupMsg {
                group: self.group.clone(),
                data: "".to_string(),
                bytes: bin,
            })),
            Ok(Message::Close(reason)) => ctx.close(reason),
            _ => (),
        }
    }
}

impl Handler<Arc<GroupMsg>> for WsConn {
    type Result = ();

    fn handle(&mut self, gm: Arc<GroupMsg>, ctx: &mut Self::Context) -> Self::Result {
        if self.group == gm.group {
            if gm.data.len() > 0 {
                ctx.text(gm.data.clone());
            }
            if gm.bytes.len() > 0 {
                ctx.binary(gm.bytes.clone())
            }
        }
    }
}

#[inline]
fn get_item() -> Option<QueueItem> {
    if QUEUE.read().unwrap().is_empty() {
        return None;
    }
    QUEUE.write().unwrap().pop()
}

fn tidy_it(res: &mut Value, v: &QueueItem) {
    res["time"] = serde_json::json!(util::unix_time());
    res["ip"] = Value::String(v.ip.clone());
    res["ua"] = Value::String(v.ua.clone());
    res["refer"] = match res.get("refer") {
        Some(rr) => match rr.as_str() {
            Some(vv) => {
                if vv.len() > 0 {
                    Value::String(vv.to_string())
                } else {
                    Value::String(v.refer.clone())
                }
            }
            None => Value::String(v.refer.clone()),
        },
        None => Value::String(v.refer.clone()),
    };
}

pub async fn taskloop(store: Arc<DbConnection>) {
    loop {
        let item = get_item();
        if item.is_none() {
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
        let v = item.unwrap();
        USERS.each(v.data.clone());
        if !store.ok() {
            tokio::time::sleep(Duration::from_millis(10)).await;
            continue;
        }
        if v.data.data.len() > 0 && v.data.data.len() < 8192 {
            let j: Result<Value, serde_json::Error> = serde_json::from_str(&v.data.data);
            if let Ok(mut res) = j {
                if res.is_object() {
                    tidy_it(&mut res, &v);
                    store.save(&v.data.group, res).await;
                }
            }
        }
        if v.data.bytes.len() > 0 && v.data.bytes.len() < 8192 {
            let j: Result<Value, serde_json::Error> = serde_json::from_slice(&v.data.bytes);
            if let Ok(mut res) = j {
                if res.is_object() {
                    tidy_it(&mut res, &v);
                    store.save(&v.data.group, res).await;
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}
