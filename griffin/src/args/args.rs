use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Deserialize, Serialize)]
#[command(name = "server", about = "Run the server with options")]
pub struct Args {
    #[arg(
        short,
        alias = "c",
        default_value = "default_config.yaml",
        help = "Configuration file path"
    )]
    pub config_path: String,
}
