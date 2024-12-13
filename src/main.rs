use std::{collections::HashMap, process::Stdio, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::ChildStdout,
    sync::Mutex,
};

mod config;
mod http;
mod journal;

struct UserLine(String, bool);

impl UserLine {
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

struct User {
    online: bool,
    online_since: Option<String>,
}

struct OnlineUsers {
    users: HashMap<String, User>,
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
        let mut lines = vec![];

        for (nick, user) in &self.users {
            if !user.online {
                continue;
            };
            lines.push(format!(
                "[{}] {nick}",
                user.online_since
                    .as_ref()
                    .expect("Online users should have online_since set")
            ))
        }

        // Header width
        {
            let mut max_width: usize = 18;
            lines.iter().for_each(|line| {
                let chars_count = line.chars().count();
                if chars_count > max_width {
                    max_width = chars_count;
                }
            });

            const TITLE: &str = "Online users";
            let title_padding = max_width - TITLE.chars().count();
            let timestamp_padding = max_width - self.updated_at.chars().count();

            let title_line = {
                let mut s = TITLE.to_string();
                s.insert_str(0, &"-".repeat(title_padding / 2));
                s.push_str(&"-".repeat(title_padding / 2));
                s
            };

            let timestamp_line = {
                let mut s = self.updated_at.clone();
                s.insert_str(0, &" ".repeat(timestamp_padding / 2));
                s.push_str(&" ".repeat(timestamp_padding / 2));
                s
            };

            lines.insert(0, title_line);
            lines.insert(1, timestamp_line);
            lines.insert(2, "-".repeat(max_width));
        }

        lines.join("\n")
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
            if let Some(user_line) = UserLine::parse(&line) {
                let parts = line.splitn(4, ' ').collect::<Vec<&str>>();
                let timestamp = format!("{} {} {}", parts[2], parts[1], parts[0]);

                let online = user_line.1;
                let online_since = if online {
                    Some(timestamp.clone())
                } else {
                    None
                };

                let user = User {
                    online,
                    online_since,
                };

                let mut lock = online_users.lock().await;
                lock.users.insert(user_line.0, user);
                lock.updated_at = timestamp;

                let userlist = lock.get_status_display();
                println!("{userlist}");
            }
        }
    })
}
