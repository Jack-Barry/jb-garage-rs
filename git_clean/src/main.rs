use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use dirs::home_dir;
use git2::{
    Branch, BranchType, Cred, Direction, Error as Git2Error, ErrorCode, PushOptions,
    RemoteCallbacks, Repository,
};
use serde_derive::Deserialize;
use std::{
    env::{self, consts::OS},
    fs::File,
    io::Read,
    path::PathBuf,
};

fn main() -> Result<()> {
    let cwd = env::current_dir().with_context(|| "Failed to determine cwd")?;
    let repo = Repository::open(cwd).with_context(|| "Failed to open repo")?;
    let branches = repo
        .branches(Some(BranchType::Local))
        .with_context(|| "Failed to get branches")?;

    let default_branch_name = get_default_branch(&repo)?;

    branches.for_each(
        |branch| match handle_branch(&repo, &default_branch_name, branch) {
            Err(error) => {
                println!("Error encountered handling branch: {}", error)
            }
            _ => (),
        },
    );
    Ok(())
}

fn prompt_user_for_delete(name: &str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Do you want to delete {}?", name))
        .interact()
        .unwrap_or(false)
}

fn get_default_branch(repo: &Repository) -> Result<String> {
    let mut remote = repo
        .find_remote("origin")
        .with_context(|| "Unable to find remote")?;

    remote
        .connect(Direction::Fetch)
        .with_context(|| "Unable to connect to remote")?;

    let default_branch_result = remote.default_branch();

    remote
        .disconnect()
        .unwrap_or_else(|error| println!("Unable to disconnect from remote: {}", error));

    let default_branch = default_branch_result?;

    let default_branch_str = String::from_utf8(default_branch.to_vec())?;
    Ok(default_branch_str)
}

fn handle_branch(
    repo: &Repository,
    default_branch: &String,
    branch: Result<(Branch, BranchType), Git2Error>,
) -> Result<()> {
    let mut verified_branch = branch.with_context(|| "Unable to use branch")?;

    if verified_branch.0.is_head() {
        println!("Skipping current branch");
        return Ok(());
    }

    if let Some(branch_name_str) = verified_branch.0.name()? {
        let default_branch_name = default_branch.replace("refs/heads/", "");
        if default_branch_name == branch_name_str {
            println!("Skipping default branch: {}", default_branch_name);
            return Ok(());
        }

        let branch_name_str_copy = branch_name_str.to_string();
        delete_local_branch(repo, &mut verified_branch, &branch_name_str_copy)?;
    };

    Ok(())
}

fn delete_local_branch(
    repo: &Repository,
    verified_branch: &mut (Branch, BranchType),
    branch_name_str: &str,
) -> Result<()> {
    let will_delete_local_branch = prompt_user_for_delete(branch_name_str);
    if will_delete_local_branch {
        handle_upstream_branch(repo, &verified_branch)
            .with_context(|| "Encountered problem handling remote branch")?;
        verified_branch
            .0
            .delete()
            .with_context(|| "Encountered problem deleting local branch")?;
    }

    Ok(())
}

fn handle_upstream_branch(repo: &Repository, branch: &(Branch, BranchType)) -> Result<()> {
    if let Some(branch_ref) = branch.0.get().name() {
        let remote = match repo.branch_upstream_remote(branch_ref) {
            Ok(r) => r,
            Err(error) => match error.code() {
                // Expected for some branches not to have a remote counterpart
                ErrorCode::NotFound => {
                    return Ok(());
                }
                _ => {
                    return Err(anyhow::Error::from(error));
                }
            },
        };

        if let Some(remote_str) = remote.as_str() {
            return delete_upstream_branch(repo, branch_ref, remote_str);
        }
    }

    Ok(())
}

fn delete_upstream_branch(repo: &Repository, branch_ref: &str, remote_str: &str) -> Result<()> {
    let will_delete_upstream_branch = prompt_user_for_delete(branch_ref);
    if will_delete_upstream_branch {
        let mut refspec: String = ":".to_owned();
        refspec.push_str(branch_ref);
        let refspecs = &[refspec];

        let mut push_options = PushOptions::new();
        let mut remote_callbacks = RemoteCallbacks::new();

        remote_callbacks.credentials(|_, _, _| match get_gh_cli_auth() {
            Ok(gh_cli) => Cred::userpass_plaintext(&gh_cli.user, &gh_cli.oauth_token),
            Err(e) => {
                eprintln!("GitHub CLI auth error: {}", e);
                Cred::default()
            }
        });
        push_options.remote_callbacks(remote_callbacks);

        let mut repo_remote = repo
            .find_remote(remote_str)
            .with_context(|| "Unable to find remote")?;

        repo_remote
            .push(refspecs, Some(&mut push_options))
            .with_context(|| "Encountered trouble deleting remote branch")?;
    }

    Ok(())
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

fn get_gh_cli_auth() -> Result<NestedGhCliConfig> {
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
