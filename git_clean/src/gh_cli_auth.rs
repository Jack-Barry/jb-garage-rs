use std::{
    env::{self, consts::OS},
    fs::File,
    io::Read,
    path::PathBuf,
};

use anyhow::{Context, Result};
use dirs::home_dir;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
struct GhCliConfig {
    #[serde(rename = "github.com")]
    github_com: NestedGhCliConfig,
}

#[derive(Debug, Deserialize)]
pub struct NestedGhCliConfig {
    #[serde(rename = "user")]
    pub user: String,
    #[serde(rename = "oauth_token")]
    pub oauth_token: String,
    // git_protocol: String,
}

pub fn get_gh_cli_auth() -> Result<NestedGhCliConfig> {
    let config_path = get_gh_cli_config_path();
    let mut config_content = String::new();

    let mut file =
        File::open(config_path).with_context(|| "Failed to open GitHub auth config file")?;

    file.read_to_string(&mut config_content)
        .with_context(|| "Failed to read GitHub auth config file contents")?;

    let yaml = serde_yaml::from_str::<GhCliConfig>(&config_content)
        .with_context(|| "Unable to parse GitHub auth config file")?;

    Ok(yaml.github_com)
}

fn get_gh_cli_config_path() -> PathBuf {
    match OS {
        "windows" => {
            let appdata_dir = env::var("APPDATA").expect("Failed to determine APPDATA directory");
            gh_config_path_from_base(PathBuf::from(appdata_dir))
        }
        _ => {
            let home_dir = home_dir().expect("Failed to determine home directory");
            gh_config_path_from_base(home_dir.join(".config"))
        }
    }
}

fn gh_config_path_from_base(base: PathBuf) -> PathBuf {
    base.join("gh").join("hosts.yml")
}
