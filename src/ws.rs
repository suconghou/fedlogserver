use std::{collections::HashMap, sync::RwLock};
use actix::{Addr,Actor, StreamHandler};
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};


lazy_static! {
    static ref USERS: Conns= Conns::new();
}

pub struct Conns {
    data: RwLock<HashMap<u64, WebsocketContext<WsConn> >>
}

impl Conns {
    pub fn new()->Self{
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
    pub fn insert(&self,id:u64,conn: WebsocketContext<WsConn>){
        let mut u = self.data.write().unwrap();
        u.insert(id, conn);
    }

    pub fn remove(&self,id:u64){
        let mut u = self.data.write().unwrap();
        u.remove(&id);
    }
    pub fn each(&self,){
        let u = self.data.read().unwrap();
        for (id,item) in u.iter() {
            
        }
    }
}

pub struct WsConn {
    pub id: u64,
    pub group :String,
}

impl Actor for WsConn {
    type Context = WebsocketContext<Self>;
    /// 连接上
    fn started(&mut self, c: &mut Self::Context) {
        println!("{} join!", self.id);
        USERS.insert(self.id, None);
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
            Ok(Message::Ping(msg)) => ctx.pong(&msg),
            Ok(Message::Text(text)) => ctx.text(text),
            Ok(Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}