use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use multimint::fedimint_core::api::InviteCode;

pub struct Config {
    pub homeserver_url: String,
    pub username: String,
    pub password: String,
    pub data_dir: PathBuf,
    pub multimint_path: PathBuf,
    pub session_file: PathBuf,
    pub default_federation_invite_code: InviteCode,
}

impl Config {
    fn from_cli(cli: Cli) -> Result<Self, anyhow::Error> {
        let multimint_path = cli.client_data_path.join("multimint");
        let session_file = cli.client_data_path.join("matrix").join("session.json");
        let default_federation_invite_code =
            InviteCode::from_str(&cli.default_federation_invite_code)?;
        Ok(Self {
            homeserver_url: cli.homeserver_url,
            username: cli.username,
            password: cli.password,
            data_dir: cli.client_data_path,
            multimint_path,
            session_file,
            default_federation_invite_code,
        })
    }
}

#[derive(Parser)]
#[clap(version = "1.0", author = "Kody Low <kodylow7@gmail.com>")]
pub struct Cli {
    /// URL of the homeserver
    #[clap(long, env = "MATRIX_HOMESERVER_URL", required = true)]
    pub homeserver_url: String,

    /// Username for login
    #[clap(long, env = "MATRIX_USERNAME", required = true)]
    pub username: String,

    /// Password for login
    #[clap(long, env = "MATRIX_PASSWORD", required = true)]
    pub password: String,

    /// Path to the client_data directory
    #[clap(long, env = "CLIENT_DATA_PATH", required = true)]
    pub client_data_path: PathBuf,

    /// Default federation invite code
    #[clap(long, env = "DEFAULT_FEDERATION_INVITE_CODE", required = true)]
    pub default_federation_invite_code: String,
}

pub fn get_config() -> Result<Config, anyhow::Error> {
    dotenv::dotenv().ok();
    let cli = Cli::parse();
    Config::from_cli(cli)
}
