use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::main]
async fn main() {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:3000".to_string());
    println!("{}", addr);
    let stream = TcpStream::connect(&addr).await.expect("connection failed");
    let (mut reader, mut writer) = io::split(stream);

    println!("Connected to {:?}", addr);
    loop {
        let mut cmd_txt_buf = String::new();
        match std::io::stdin().read_line(&mut cmd_txt_buf) {
            Ok(n) => {
                if n == 1 {
                    continue;
                }
            }
            Err(e) => {
                println!("cannot parse text: {:?}", e);
                continue;
            }
        }

        cmd_txt_buf.remove(cmd_txt_buf.len() - 1);

        writer
            .write_all(cmd_txt_buf.as_bytes())
            .await
            .expect("couldn't send to server");

        let mut server_buf = [0; 1024];
        match reader.read(&mut server_buf).await {
            Ok(n) => {
                if n == 0 {
                    println!("connection closed by server");
                    break;
                }
                let data = &server_buf[..n];
                let res_body = String::from_utf8(data.to_vec()).expect("couldn't read from server");
                println!(">> {}", res_body);
            }
            Err(e) => {
                println!("error reading from server: {:?}", e);
                break;
            }
        }
    }
}

// exsample
// REQUEST  -> {"header":{"command":"SET","key":"hello"},"body":"world"}
// RESPONSE -> {"statuscode":204,"header":{"comment":"Set value"},"body":"OK"}
//
// REQUEST  -> {"header":{"command":"GET","key":"hello"},"body":null}
// RESPONSE -> {"statuscode":200,"header":{"comment":"Get value"},"body":"world"}
