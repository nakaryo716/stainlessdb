use hashbrown::HashMap;
use stainlessdb_core::cmd::{
    response::{ResJsonCmd, ResponseCmdUtil},
    DbCmd,
};
use std::{future::Future, pin::Pin};
use string_type::StringType;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};
use tracing::warn;

mod string_type;

struct Database {
    string_pool: HashMap<String, String>,
}

impl Database {
    fn new() -> Self {
        Database {
            string_pool: HashMap::default(),
        }
    }
}

pub struct DbChannel {}

impl DbChannel {
    pub fn new(buffer: usize) -> (Sender<DbCmd>, Receiver<DbCmd>) {
        mpsc::channel(buffer)
    }
}

// チャンネルを使用したアクターモデルでデータベース操作を行う
// Sender<DbCmd>でコマンドが送られてくる
pub fn run_db_actor(mut rx: Receiver<DbCmd>) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
    let mut db = Database::new();
    Box::pin(async move {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                DbCmd::Set { json_cmd, sender } => {
                    // DbCmd::Setが作成された時点でbodyは必ずSomeであることが保証されているため.unrap()できる
                    db.set(&json_cmd.get_key(), &json_cmd.get_body().unwrap());

                    let response = ResJsonCmd::ok();
                    send_res_cmd(sender, response);
                }
                DbCmd::Get { json_cmd, sender } => {
                    let response = match db.get(&json_cmd.get_key()) {
                        Some(body) => ResJsonCmd::ok_with_body(&body),
                        None => ResJsonCmd::key_not_found(),
                    };
                    send_res_cmd(sender, response);
                }
                DbCmd::Del { json_cmd, sender } => {
                    db.delete(&json_cmd.get_key());
                    send_res_cmd(sender, ResJsonCmd::ok());
                }
                DbCmd::NotFound { sender } => {
                    send_res_cmd(sender, ResJsonCmd::cmd_not_found());
                }
            }
        }
    })
}

// クライアントにレスポンスJsonコマンドを送信する
// 送信失敗してもパニックしない
fn send_res_cmd(sender: oneshot::Sender<ResJsonCmd>, response: ResJsonCmd) {
    if let Err(e) = sender.send(response) {
        warn!("{:?}", e);
    }
}
