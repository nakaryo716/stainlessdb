use serde::{Deserialize, Serialize};

// サーバーがクライアントに送信するコマンド情報
// レスポンス型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResJsonCmd {
    statuscode: u32,
    header: ResHeader,
    body: Option<ResBody>,
}

impl ResJsonCmd {
    pub fn new(statuscode: u32, header: ResHeader, body: Option<ResBody>) -> Self {
        Self {
            statuscode,
            header,
            body,
        }
    }
}

// ResJsonCmdに対するユーティリティ
// よく使うメソッドを用意する
pub trait ResponseCmdUtil {
    type Output;
    fn ok() -> Self::Output;
    fn ok_with_body(body: &str) -> Self::Output;
    fn key_not_found() -> Self::Output;
    fn cmd_not_found() -> Self::Output;
}

impl ResponseCmdUtil for ResJsonCmd {
    type Output = Self;

    fn ok() -> Self {
        Self {
            statuscode: 204,
            header: ResHeader {
                comment: "Command success".to_string(),
            },
            body: Some(ResBody::new("OK")),
        }
    }
    fn ok_with_body(body: &str) -> Self {
        Self {
            statuscode: 200,
            header: ResHeader {
                comment: "Get value".to_string(),
            },
            body: Some(ResBody::new(body)),
        }
    }
    fn key_not_found() -> Self {
        Self {
            statuscode: 400,
            header: ResHeader {
                comment: "Key not found".to_string(),
            },
            body: None,
        }
    }
    fn cmd_not_found() -> Self {
        Self {
            statuscode: 404,
            header: ResHeader {
                comment: "Command not found".to_string(),
            },
            body: None,
        }
    }
}

// ヘッダー情報を提供する型
// ユーザーに対してメッセージを残す
// 例
// 1. keyが見つからない
// 2. コマンドが違う
//
// タイムスタンプの追加予定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResHeader {
    comment: String,
}

impl ResHeader {
    pub fn new(comment: &str) -> Self {
        Self {
            comment: comment.to_string(),
        }
    }
}

// クエリ結果のラッパー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResBody(String);

impl ResBody {
    pub fn new(value: &str) -> Self {
        Self(value.to_string())
    }
}
