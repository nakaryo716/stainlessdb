use request::ReqJsonCmd;
use response::ResJsonCmd;
use tokio::sync::oneshot::Sender;

pub mod request;
pub mod response;

// サーバーに送る最終的なコマンド型
// チャンネルを使用してサーバーにデータを送る"mpsc::Sender<DbCmd>"
//
// oneshot::Sender<ResJsonCmd>をDbCmdと一緒に送り
// サーバーからのデータはoneshot::Reciever<ResJsonCmd>で受け取る
#[derive(Debug)]
pub enum DbCmd {
    Set {
        json_cmd: ReqJsonCmd,
        sender: Sender<ResJsonCmd>,
    },
    Get {
        json_cmd: ReqJsonCmd,
        sender: Sender<ResJsonCmd>,
    },
    Del {
        json_cmd: ReqJsonCmd,
        sender: Sender<ResJsonCmd>,
    },
    NotFound {
        sender: Sender<ResJsonCmd>,
    },
}

// クライアントからのJsonを受け取ってDbCmdを作成する
pub fn db_cmd_producer(json_cmd: ReqJsonCmd, sender: Sender<ResJsonCmd>) -> DbCmd {
    match json_cmd.get_header().get_command() {
        "SET" if json_cmd.get_body().is_some() => DbCmd::Set { json_cmd, sender },
        "GET" if json_cmd.get_body().is_none() => DbCmd::Get { json_cmd, sender },
        "DEL" if json_cmd.get_body().is_none() => DbCmd::Del { json_cmd, sender },
        &_ => DbCmd::NotFound { sender },
    }
}
