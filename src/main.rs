use std::{collections::HashMap, process::Stdio, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::ChildStdout,
    sync::Mutex,
};

mod config;
mod http;

struct User(String, bool);

impl User {
    pub fn parse(s: &str) -> Option<Self> {
        if s.contains("|INFO")
            && (s.contains("|client connected '") || s.contains("|client disconnected '"))
        {
            let online = s.contains("|client connected '");

            let username = if online {
                s.split_once("|client connected '").unwrap().1
            } else {
                s.split_once("|client disconnected '").unwrap().1
            };

            let username = username.split_once("'(id:").unwrap().0;

            Some(Self(username.to_string(), online))
        } else {
            None
        }
    }
}

struct OnlineUsers {
    users: HashMap<String, bool>,
    updated_at: String,
}
type ArcOnlineUsers = Arc<Mutex<OnlineUsers>>;

impl OnlineUsers {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            updated_at: String::new(),
        }
    }
    pub fn get_status_display(&self) -> String {
        let mut lines = vec![
            "-----Online users-----".to_string(),
            format!("   {}   ", self.updated_at),
            "----------------------".to_string(),
        ];

        for (user, online) in &self.users {
            if !online {
                continue;
            };
            lines.push(user.to_string())
        }

        lines.join("\n")
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::parse_args();

    let mut journalctl_command = tokio::process::Command::new("journalctl");
    journalctl_command.args(["-f", "-u", &config.unit, "--no-pager"]);
    if let Some(since) = config.since.as_ref() {
        journalctl_command.arg("--since");
        journalctl_command.arg(since);
    }

    let mut journalctl_process = journalctl_command
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let stdout = journalctl_process.stdout.take().unwrap();

    let online_users: ArcOnlineUsers = Arc::new(Mutex::new(OnlineUsers::new()));

    let parser_handle = parse_logs(stdout, online_users.clone());

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

fn parse_logs(stdout: ChildStdout, online_users: ArcOnlineUsers) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut logs = BufReader::new(stdout).lines();

        while let Some(line) = logs.next_line().await.unwrap() {
            if let Some(user) = User::parse(&line) {
                let parts = line.splitn(4, ' ').collect::<Vec<&str>>();
                let timestamp = format!("{} {} {}", parts[0], parts[1], parts[2]);

                let mut lock = online_users.lock().await;
                lock.users.insert(user.0, user.1);
                lock.updated_at = timestamp;

                let userlist = lock.get_status_display();
                println!("{userlist}");
            }
        }
    })
}
