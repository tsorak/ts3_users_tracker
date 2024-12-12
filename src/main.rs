use std::{collections::HashMap, process::Stdio, sync::Arc};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::ChildStdout,
    sync::Mutex,
};

struct User(String, bool);

impl User {
    pub fn parse(s: &String) -> Option<Self> {
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

struct OnlineUsers(HashMap<String, bool>);
type ArcOnlineUsers = Arc<Mutex<OnlineUsers>>;

impl OnlineUsers {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn print_status(&self, timestamp: &str) {
        println!();
        println!("-----Online users-----");
        println!("   {timestamp}   ");
        println!("----------------------");

        for (user, online) in &self.0 {
            if !online {
                continue;
            };
            println!("{user}");
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut journalctl_process = tokio::process::Command::new("journalctl")
        .args([
            "-f",
            "-u",
            "teamspeak3-server.service",
            "--no-pager",
            "--since",
            "-3d",
        ])
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;

    let stdout = journalctl_process.stdout.take().unwrap();

    let online_users: ArcOnlineUsers = Arc::new(Mutex::new(OnlineUsers::new()));

    let parser_handle = parse_logs(stdout, online_users.clone());

    let _ = parser_handle.await;

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
                lock.0.insert(user.0, user.1);

                lock.print_status(&timestamp);
            }
        }
    })
}
