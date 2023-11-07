use clap::Parser;

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub run: RunConfig,
}

impl Config {
    pub fn new() -> Self {
        let args = Args::parse();

        Self {
            run: RunConfig {
                host: args.host,
                port: args.port,
            },
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, Parser)]
struct Args {
    #[clap(long, env = "HOST", default_value = "0.0.0.0")]
    host: String,

    #[clap(short, long, env = "PORT", default_value = "3000")]
    port: u16,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RunConfig {
    pub host: String,
    pub port: u16,
}
