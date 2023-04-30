use std::{
    env, fs, io,
    path::{self, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::from_str;

fn main() {
    let working_dir = env::current_dir();

    match working_dir {
        Ok(cwd) => get_config_from_dir(cwd),
        Err(e) => {
            println!("{:?}", e)
        }
    }
}

fn get_config_from_dir(dir: PathBuf) {
    let mut config_path = dir;
    config_path.push("versionerrc.json");
    match config_path.to_str() {
        Some(path) => {
            handle_config_path(path);
        }
        _ => {
            println!("No path to read config from")
        }
    }
}

fn handle_config_path(path: &str) {
    let config = read_config(path);
    match config {
        Ok(mut cfg) => {
            cfg.merge(Some(VersionerConfig::default()));
            println!("{:?}", cfg);
        }
        Err(e) => {
            println!("Problem with config: {:?}", e)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct VersionerConfig {
    projects: Option<String>,
    other_field: Option<String>,
}

impl Default for VersionerConfig {
    fn default() -> Self {
        VersionerConfig {
            projects: None,
            other_field: Some(String::from("default-other-field")),
        }
    }
}

impl VersionerConfig {
    fn merge(&mut self, other_config: Option<VersionerConfig>) {
        match other_config {
            Some(config) => {
                // projects
                let self_projects = self.projects.take().unwrap_or_default();
                let merged_projects = config.projects.or_else(|| Some(self_projects));
                self.projects = merged_projects;
                // other_field
                let self_other_field = self.other_field.take().unwrap_or_default();
                let merged_other_field = config.other_field.or_else(|| Some(self_other_field));
                self.other_field = merged_other_field;
            }
            None => {}
        }
    }
}

/// Given a config path, returns a `VersionerConfig` object
///
/// If no config is present, returns a default `VersionerConfig`
fn read_config(config_path: &str) -> Result<VersionerConfig, io::Error> {
    if !file_or_dir_exists(config_path) {
        return Ok(VersionerConfig::default());
    }

    let config_contents = fs::read_to_string(config_path)?;
    let config_json: VersionerConfig = from_str(&config_contents)?;
    Ok(config_json)
}

/// Determines if entity exists at provided path
fn file_or_dir_exists(path: &str) -> bool {
    let entity = path::Path::new(path);
    entity.exists()
}
