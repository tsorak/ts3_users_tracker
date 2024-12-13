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
        help = "How far back to read logs from. (Defaults to unit start date)\nSee journalctl '--since' for appropriate formatting"
    )]
    pub since: Option<String>,
}

pub async fn parse_args() -> Config {
    let mut cfg = Config::parse();

    if cfg.since.is_none() {
        if let Ok(unit_start_date) = crate::journal::get_unit_start_date(&cfg).await {
            let _ = cfg.since.insert(unit_start_date);
        };
    }

    cfg
}
