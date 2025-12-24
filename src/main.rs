mod protocol;
mod semgrep_wrapper;
mod api_client;
mod handler;

use anyhow::Result;
use clap::Parser;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info};
use protocol::{JsonRpcMessage, JsonRpcResponse, JsonRpcErrorResponse, JsonRpcError};
use handler::Handler;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // We can add args like --port later for HTTP support
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr to avoid corrupting stdout JSON-RPC
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let _args = Args::parse();

    info!("Starting semgrep-mcp-rs server...");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    
    let mut reader = BufReader::new(stdin);
    let mut writer = stdout;

    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break; // EOF
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match serde_json::from_str::<JsonRpcMessage>(trimmed) {
            Ok(msg) => {
                match msg {
                    JsonRpcMessage::Request(req) => {
                        let id = req.id.clone();
                        match Handler::handle_request(req).await {
                            Ok(result) => {
                                let response = JsonRpcMessage::Response(JsonRpcResponse {
                                    jsonrpc: "2.0".to_string(),
                                    id,
                                    result,
                                });
                                send_message(&mut writer, &response).await?;
                            }
                            Err(err) => {
                                let response = JsonRpcMessage::Error(JsonRpcErrorResponse {
                                    jsonrpc: "2.0".to_string(),
                                    id: Some(id),
                                    error: err,
                                });
                                send_message(&mut writer, &response).await?;
                            }
                        }
                    }
                    JsonRpcMessage::Notification(notif) => {
                        // Handle notifications (no response needed)
                        if notif.method == "notifications/initialized" {
                            info!("Client initialized notification received");
                        }
                    }
                     _ => {
                        // Ignore responses or errors sent TO the server for now
                    }
                }
            }
            Err(e) => {
                error!("Failed to parse JSON: {}", e);
                // Send parse error
                 let response = JsonRpcMessage::Error(JsonRpcErrorResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    error: JsonRpcError {
                        code: -32700,
                        message: "Parse error".to_string(),
                        data: None,
                    },
                });
                send_message(&mut writer, &response).await?;
            }
        }
    }

    Ok(())
}

async fn send_message<W: AsyncWriteExt + Unpin>(writer: &mut W, msg: &JsonRpcMessage) -> Result<()> {
    let json = serde_json::to_string(msg)?;
    writer.write_all(json.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;
    Ok(())
}
