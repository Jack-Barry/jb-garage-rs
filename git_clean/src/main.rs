use dialoguer::{theme::ColorfulTheme, Confirm};
use git2::{Branch, BranchType, Direction, Error, Repository};
use std::env;

fn main() {
    let cwd = match env::current_dir() {
        Ok(path) => path,
        Err(e) => panic!("Failed to determine cwd: {}", e),
    };

    let repo = match Repository::open(cwd) {
        Ok(repo) => repo,
        Err(e) => panic!("Failed to open repo: {}", e),
    };

    let default_branch_name = get_default_branch(&repo);

    let branches = match repo.branches(Some(BranchType::Local)) {
        Ok(branches) => branches,
        Err(e) => panic!("Failed to get branches: {}", e),
    };

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
            // skip the current branch
            if verified_branch.0.is_head() {
                return;
            }

            match verified_branch.0.name() {
                Ok(branch_name) => match branch_name {
                    Some(branch_name_str) => {
                        match default_branch {
                            Some(default_branch_str) => {
                                // Skip deleting the default branch
                                if default_branch_str.replace("refs/heads/", "") == branch_name_str
                                {
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

        match repo.find_remote(remote_str) {
            Ok(mut repo_remote) => match repo_remote.push(refspecs, None) {
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
