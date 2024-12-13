use std::{collections::HashMap, process::Stdio, sync::Arc};

use tokio::sync::Mutex;

mod config;
mod http;
mod journal;
mod log;

type ArcOnlineUsers = Arc<Mutex<OnlineUsers>>;

struct OnlineUsers {
    users: HashMap<String, User>,
    updated_at: String,
}

struct User {
    online: bool,
    online_since: Option<String>,
}

impl OnlineUsers {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            updated_at: String::new(),
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::parse_args().await;

    let mut journalctl_process = journal::reader(&config)
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let stdout = journalctl_process.stdout.take().unwrap();

    let online_users: ArcOnlineUsers = Arc::new(Mutex::new(OnlineUsers::new()));

    let parser_handle = log::parse(stdout, online_users.clone());

    if config.serve_http {
        let http_server = http::start(online_users, config.port).await?;

        tokio::select! {
            _ = parser_handle => {},
            result = http_server => {
                if let Err(err) = result {
                    eprintln!("{}", err);
                }
            },
        }
    } else {
        let _ = parser_handle.await;
    }

    Ok(())
}
