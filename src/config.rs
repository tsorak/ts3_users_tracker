use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    #[arg(short, long, default_value = "teamspeak3-server.service")]
    pub unit: String,

    #[arg(long = "serve-http")]
    pub serve_http: bool,

    #[arg(short, long, default_value_t = 3000)]
    pub port: u16,

    #[arg(
        long,
        help = "How far back to read logs from.\nSee journalctl '--since' for appropriate formatting"
    )]
    pub since: Option<String>,
}

pub fn parse_args() -> Config {
    Config::parse()
}
