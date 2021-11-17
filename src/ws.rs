use actix::{Actor, Addr, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use std::{collections::HashMap, sync::RwLock};

use crate::queue::Queue;

lazy_static! {
    pub static ref USERS: Conns = Conns::new();
    pub static ref QUEUE: RwLock<Queue<GroupMsg>> = RwLock::new(Queue::new());
}

pub struct Conns {
    data: RwLock<HashMap<u64, Addr<WsConn>>>,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct GroupMsg {
    pub group: String,
    pub data: String,
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

    fn each(&self, group: &String, data: String) {
        let u = self.data.read().unwrap();
        for item in u.values() {
            item.do_send(GroupMsg {
                group: group.clone(),
                data: data.clone(),
            })
        }
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
            Ok(Message::Text(text)) => USERS.each(&self.group, text),
            Ok(Message::Ping(msg)) => ctx.pong(&msg),
            Ok(Message::Binary(bin)) => ctx.binary(bin),
            Ok(Message::Close(reason)) => ctx.close(reason),
            _ => (),
        }
    }
}

impl Handler<GroupMsg> for WsConn {
    type Result = ();

    fn handle(&mut self, gm: GroupMsg, ctx: &mut Self::Context) -> Self::Result {
        if self.group == gm.group {
            ctx.text(gm.data);
        }
    }
}
