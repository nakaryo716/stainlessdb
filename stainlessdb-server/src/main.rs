use database::{run_db_actor, DbChannel};
use stainlessdb_core::cmd::{
    db_cmd_producer,
    response::{ResHeader, ResJsonCmd, ResponseCmdUtil},
};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, WriteHalf},
    net::TcpStream,
    sync::oneshot,
};

use tracing::{info, warn};

mod database;

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Server start");

    // tcp listener binding
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    info!("Server listening on {:?}", listener);

    // this channel is used to comunicate with database task and server task
    let (sender_for_db, receiver_from_db) = DbChannel::new(128);

    // データベースタスク
    // run_db_actorは絶対にpanicしないようになっている
    // in memoryでpanicするとデータが消えるため
    tokio::task::spawn(run_db_actor(receiver_from_db));

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Connection established: {:?}", addr);

        let (mut cli_reader, mut cli_writer) = io::split(stream);
        let sender_for_db = sender_for_db.clone();

        tokio::task::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                match cli_reader.read(&mut buf).await {
                    Ok(n) => {
                        // n == 0 mean cliant send nothing
                        // We can not do anything
                        // So we closse connection
                        if n == 0 {
                            break;
                        }

                        // Get data that only cliant was writtten buffer
                        let written_data = &buf[..n];

                        // クライアントからのバイトデータを文字列にエンコード
                        // 失敗したらやり直しさせる
                        // コネクションは維持する
                        let Ok(cmd_str) = String::from_utf8(written_data.to_vec()) else {
                            // クライアントにエンコード失敗を教える
                            // クライアントへの送信に失敗したら、できることがない為、ループを抜け接続を閉じる
                            if let Err(_) = send_encode_fail_res(&mut cli_writer).await {
                                warn!("Error happened while Send encode fail response");
                                break;
                            }
                            continue;
                        };

                        // RewJsonCmd型にシリアライズ
                        // シリアライズが失敗したときはプロトコルに準拠してないコマンドなのでCommand not foundを教える
                        // コネクションは維持する
                        let Ok(json_cmd) = serde_json::from_str(&cmd_str) else {
                            // クライアントにコマンドが正しくないことを教える
                            // クライアントへの送信に失敗したら、できることがない為、ループを抜け接続を閉じる
                            if let Err(_) = send_command_not_found_res(&mut cli_writer).await {
                                warn!("Error happened while Send command not found response");
                                break;
                            }
                            continue;
                        };

                        // コマンド作成とデータベースからのレスポンスを取得するためのchannelを作成
                        let (db_fetch_sender, db_res_reciver) = oneshot::channel();
                        let db_cmd = db_cmd_producer(json_cmd, db_fetch_sender);

                        // データベースタスクへの問い合わせ
                        // 失敗したときは致命的な問題がサーバーで起きている可能性が高いためクライアントとの接続を切る
                        if let Err(e) = sender_for_db.send(db_cmd).await {
                            // クライアントにデーターベースに問題があることを教える
                            // クライアントへの送信に失敗したら、できることがない為、ループを抜け接続を閉じる
                            if let Err(_) = send_db_error(&mut cli_writer).await {
                                warn!("Error happened while Send database fetch fail response");
                                break;
                            };
                            warn!("{:?}", e);
                            break;
                        }

                        // データーベースタスクからの結果を取得する
                        // 失敗したときは致命的な問題がサーバーで起きている可能性が高いためクライアントとの接続を切る
                        let Ok(response) = db_res_reciver.await else {
                            // クライアントにデーターベースに問題があることを教える
                            // クライアントへの送信に失敗したら、できることがない為、ループを抜け接続を閉じる
                            if let Err(_) = send_db_error(&mut cli_writer).await {
                                warn!("Error happened while Send database fetch fail response");
                                break;
                            }
                            break;
                        };

                        // データーベースからの結果を文字列にエンコード
                        // 基本的に失敗することはあり得ない
                        // ここで失敗するなら、この先のリクエストも失敗する可能性が高いためコネクションは切断する
                        let Ok(des_response) = serde_json::to_string(&response) else {
                            // クライアントにエンコード失敗を教える
                            // クライアントへの送信に失敗したら、できることがない為、ループを抜け接続を閉じる
                            if let Err(_) = send_encode_fail_res(&mut cli_writer).await {
                                warn!("Error happened while Send encode fail response");
                                break;
                            }
                            break;
                        };

                        // 文字列をバイト列に変換
                        let encoded_txt = des_response.as_bytes();

                        // バッファに書き込む
                        // 失敗したらできることがないためタスクを終了させる
                        if cli_writer.write_all(&encoded_txt).await.is_err() {
                            warn!("Write buffer Error: {:?}", addr);
                            break;
                        }
                    }
                    Err(_e) => {
                        warn!("Rerad buffer Error: {:?}", addr);
                        break;
                    }
                }
            }
            info!("Connection closed: {:?}", addr);
        });
    }
}

struct SendError {}
// エンコードに失敗したことをクライアントに教える
async fn send_encode_fail_res(writer: &mut WriteHalf<TcpStream>) -> Result<(), SendError> {
    let ser_res = gen_encode_fail_res();
    writer
        .write_all(ser_res.as_bytes())
        .await
        .map_err(|_e| SendError {})
}

// コマンドが正しくないことをクライアントに教える
async fn send_command_not_found_res(writer: &mut WriteHalf<TcpStream>) -> Result<(), SendError> {
    let ser_res = gen_cmd_not_found_res();
    writer
        .write_all(ser_res.as_bytes())
        .await
        .map_err(|_e| SendError {})
}

// サーバーのデータベースシステムに問題があって失敗したことをクライアントに教える
async fn send_db_error(writer: &mut WriteHalf<TcpStream>) -> Result<(), SendError> {
    let ser_res = gen_db_error_res();
    writer
        .write_all(ser_res.as_bytes())
        .await
        .map_err(|_e| SendError {})
}

//　エンコードに失敗したことを教えるためのコマンドを生成する
// 失敗することはないため.unwrapを使っている
fn gen_encode_fail_res() -> String {
    let res = ResJsonCmd::new(400, ResHeader::new("failed to encode text"), None);
    serde_json::to_string(&res).unwrap()
}

// コマンドが正しくないことを教えるためのコマンドを生成する
// 失敗することはないため.unwrapを使っている
fn gen_cmd_not_found_res() -> String {
    let res = ResJsonCmd::cmd_not_found();
    serde_json::to_string(&res).unwrap()
}

//　データベースとのやりとりに失敗したことを教えるためのコマンドを生成する
// 失敗することはないため.unwrapを使っている
fn gen_db_error_res() -> String {
    let res = ResJsonCmd::new(
        500,
        ResHeader::new("failed to access database server"),
        None,
    );
    serde_json::to_string(&res).unwrap()
}

#[cfg(test)]
mod test {
    use stainlessdb_core::cmd::response::{ResHeader, ResJsonCmd};

    use crate::gen_encode_fail_res;

    #[test]
    fn test_gen_encode_fail_res() {
        let generated_txt = gen_encode_fail_res();

        let verify_res_cmd = ResJsonCmd::new(400, ResHeader::new("failed to encode text"), None);
        let verify_res_txt = serde_json::to_string(&verify_res_cmd).unwrap();
        assert_eq!(verify_res_txt, generated_txt);
    }
}
