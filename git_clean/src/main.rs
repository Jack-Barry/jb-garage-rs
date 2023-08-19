use dialoguer::{theme::ColorfulTheme, Confirm};
use dirs::home_dir;
use git2::{Branch, BranchType, Cred, Direction, Error, PushOptions, RemoteCallbacks, Repository};
use serde_derive::Deserialize;
use std::{
    env::{self, consts::OS},
    fs::File,
    io::Read,
    path::PathBuf,
};

fn main() {
    let cwd = match env::current_dir() {
        Ok(path) => path,
        Err(e) => panic!("Failed to determine cwd: {}", e),
    };

    let repo = match Repository::open(cwd) {
        Ok(repo) => repo,
        Err(e) => panic!("Failed to open repo: {}", e),
    };

    let branches = match repo.branches(Some(BranchType::Local)) {
        Ok(branches) => branches,
        Err(e) => panic!("Failed to get branches: {}", e),
    };

    let default_branch_name = get_default_branch(&repo);

    branches.for_each(|branch| handle_branch(&repo, &default_branch_name, branch));
}

fn prompt_user_for_delete(name: &str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Do you want to delete {}?", name))
        .interact()
        .unwrap()
}

fn get_default_branch(repo: &Repository) -> Option<String> {
    match repo.find_remote("origin") {
        Ok(mut remote) => match remote.connect(Direction::Fetch) {
            Ok(_) => {
                let default_branch_result = remote.default_branch();
                match remote.disconnect() {
                    Err(e) => {
                        println!("Unable to disconnect from remote: {}", e)
                    }
                    _ => {}
                };

                match default_branch_result {
                    Ok(default_branch) => match String::from_utf8(default_branch.to_vec()) {
                        Ok(default_branch_str) => Some(default_branch_str),
                        Err(e) => {
                            println!("Unable to get string from default branch: {}", e);
                            None
                        }
                    },
                    _ => None,
                }
            }
            Err(e) => {
                println!("Error connecting to remote: {}", e);
                None
            }
        },
        Err(e) => {
            println!("Error finding remote: {}", e);
            None
        }
    }
}

fn handle_branch(
    repo: &Repository,
    default_branch: &Option<String>,
    branch: Result<(Branch, BranchType), Error>,
) {
    match branch {
        Ok(mut verified_branch) => {
            if verified_branch.0.is_head() {
                println!("Skipping current branch");
                return;
            }

            match verified_branch.0.name() {
                Ok(branch_name) => match branch_name {
                    Some(branch_name_str) => {
                        match default_branch {
                            Some(default_branch_str) => {
                                let default_branch_name =
                                    default_branch_str.replace("refs/heads/", "");
                                if default_branch_name == branch_name_str {
                                    println!("Skipping default branch: {}", default_branch_name);
                                    return;
                                }
                            }
                            _ => (),
                        }
                        let branch_name_str_copy = branch_name_str.to_string();
                        delete_local_branch(repo, &mut verified_branch, &branch_name_str_copy)
                    }
                    _ => (),
                },
                Err(e) => {
                    println!("Unable to get name of branch: {}", e)
                }
            }
        }
        Err(e) => {
            println!("Unable to use branch: {}", e)
        }
    };
}

fn delete_local_branch(
    repo: &Repository,
    verified_branch: &mut (Branch, BranchType),
    branch_name_str: &str,
) {
    let will_delete_local_branch = prompt_user_for_delete(branch_name_str);
    if will_delete_local_branch {
        handle_upstream_branch(repo, &verified_branch);
        match verified_branch.0.delete() {
            Err(e) => {
                println!("Error when deleting local branch: {}", e)
            }
            _ => (),
        };
    }
}

fn handle_upstream_branch(repo: &Repository, branch: &(Branch, BranchType)) {
    match branch.0.get().name() {
        Some(branch_ref) => match repo.branch_upstream_remote(branch_ref) {
            Ok(remote) => match remote.as_str() {
                Some(remote_str) => delete_upstream_branch(repo, branch_ref, remote_str),
                _ => (),
            },
            _ => (),
        },
        _ => (),
    }
}

fn delete_upstream_branch(repo: &Repository, branch_ref: &str, remote_str: &str) {
    let will_delete_upstream_branch = prompt_user_for_delete(branch_ref);
    if will_delete_upstream_branch {
        let mut refspec: String = ":".to_owned();
        refspec.push_str(branch_ref);
        let refspecs = &[refspec];

        let mut push_options = PushOptions::new();
        let mut remote_callbacks = RemoteCallbacks::new();
        remote_callbacks.credentials(|_, _, _| {
            let gh_cli = get_gh_cli_auth();
            Cred::userpass_plaintext(&gh_cli.user, &gh_cli.oauth_token)
        });
        push_options.remote_callbacks(remote_callbacks);

        match repo.find_remote(remote_str) {
            Ok(mut repo_remote) => match repo_remote.push(refspecs, Some(&mut push_options)) {
                Err(e) => {
                    println!("Encountered trouble deleting remote branch: {}", e)
                }
                _ => (),
            },
            Err(e) => {
                println!("Failed to find remote: {}", e)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct GhCliConfig {
    #[serde(rename = "github.com")]
    github_com: NestedGhCliConfig,
}

#[derive(Debug, Deserialize)]
struct NestedGhCliConfig {
    #[serde(rename = "user")]
    user: String,
    #[serde(rename = "oauth_token")]
    oauth_token: String,
    // git_protocol: String,
}

fn get_gh_cli_auth() -> NestedGhCliConfig {
    let config_path = get_gh_cli_config_path();
    let mut config_content = String::new();

    match File::open(config_path) {
        Ok(mut file) => {
            match file.read_to_string(&mut config_content) {
                Err(e) => {
                    panic!("Failed to read config file contents: {}", e);
                }
                _ => match serde_yaml::from_str::<GhCliConfig>(&config_content) {
                    Ok(yaml) => {
                        return yaml.github_com;
                    }
                    Err(e) => {
                        panic!("Failed to parse config content: {}", e);
                    }
                },
            };
        }
        Err(e) => {
            panic!("Failed to open config file: {}", e)
        }
    }
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
