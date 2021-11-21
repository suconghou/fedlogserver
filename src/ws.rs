use actix::{Actor, Addr, AsyncContext, Handler, StreamHandler};
use actix_web::web::Bytes;
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use std::{thread, time::Duration};

use crate::queue::Queue;

lazy_static! {
    pub static ref USERS: Conns = Conns::new();
    pub static ref QUEUE: RwLock<Queue<Arc<QueueItem>>> = RwLock::new(Queue::new());
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
    pub data: GroupMsg,
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

    fn each(&self, group: &String, data: String, bin: Bytes) {
        let u = self.data.read().unwrap();
        let msg = Arc::new(GroupMsg {
            group: group.clone(),
            data,
            bytes: bin,
        });
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
        println!("{} join!", self.id);
        USERS.insert(self.id, ctx.address());
    }

    /// 断开连接
    fn stopped(&mut self, _: &mut Self::Context) {
        println!("{} exit!", self.id);
        USERS.remove(self.id);
    }
}

/// Handler for Message message
impl StreamHandler<Result<Message, ProtocolError>> for WsConn {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(Message::Text(text)) => USERS.each(&self.group, text, Bytes::new()),
            Ok(Message::Ping(msg)) => ctx.pong(&msg),
            Ok(Message::Binary(bin)) => USERS.each(&self.group, "".to_owned(), bin),
            Ok(Message::Close(reason)) => ctx.close(reason),
            _ => (),
        }
    }
}

impl Handler<Arc<GroupMsg>> for WsConn {
    type Result = ();

    fn handle(&mut self, gm: Arc<GroupMsg>, ctx: &mut Self::Context) -> Self::Result {
        if self.group == gm.group {
            if gm.data != "" {
                ctx.text(gm.data.clone());
            }
            if gm.bytes.len() > 0 {
                ctx.binary(gm.bytes.clone())
            }
        }
    }
}

fn get_item() -> Option<Arc<QueueItem>> {
    if QUEUE.read().unwrap().is_empty() {
        return None;
    }
    QUEUE.write().unwrap().pop()
}

pub fn taskloop() {
    loop {
        if let Some(v) = get_item() {
            USERS.each(&v.data.group, v.data.data.clone(), v.data.bytes.clone())
            // TODO parse json & add ua , ip, check refer & save db
        } else {
            thread::sleep(Duration::from_secs(1))
        }
        thread::sleep(Duration::from_millis(1))
    }
}
