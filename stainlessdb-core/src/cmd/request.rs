use serde::{Deserialize, Serialize};

// クライアントから送られてくるJsonコマンド
// このJsonを通して、データの追加、クエリなどを行う
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReqJsonCmd {
    header: ReqHeader,
    body: Option<ReqBody>,
}

impl ReqJsonCmd {
    pub fn get_key(&self) -> String {
        self.header.key.clone()
    }

    pub fn get_body(&self) -> Option<String> {
        self.body.clone().map(|e| e.0)
    }

    pub fn get_header(&self) -> ReqHeader {
        self.header.clone()
    }

    pub fn new(header: ReqHeader, body: Option<ReqBody>) -> Self {
        Self {
            header: header,
            body,
        }
    }
}

// データベースへの操作を指定する
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReqHeader {
    // SET, GET, DElなどのコマンド群
    command: String,
    // 各コマンドで操作する値を指定するためのkey
    key: String,
}

impl ReqHeader {
    pub fn get_command(&self) -> &str {
        &self.command
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }
}

impl ReqHeader {
    pub fn new(command: &str, key: &str) -> Self {
        Self {
            command: command.to_string(),
            key: key.to_string(),
        }
    }
}

// リクエストのValueに相当する
// SETコマンドで使用する
// その他はOption::Noneで指定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReqBody(String);

impl ReqBody {
    pub fn new(value: &str) -> Self {
        ReqBody(value.to_string())
    }
}
