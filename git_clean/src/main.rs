use dialoguer::{theme::ColorfulTheme, Confirm};
use git2::{Branch, BranchType, Error, Repository};
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

    let branches = match repo.branches(Some(BranchType::Local)) {
        Ok(branches) => branches,
        Err(e) => panic!("Failed to get branches: {}", e),
    };

    branches.for_each(|branch| handle_branch(&repo, branch));
}

fn prompt_user_for_delete(name: &str) -> bool {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Do you want to delete {}?", name))
        .interact()
        .unwrap()
}

fn handle_branch(repo: &Repository, branch: Result<(Branch, BranchType), Error>) {
    match branch {
        Ok(mut verified_branch) => match verified_branch.0.name() {
            Ok(branch_name) => match branch_name {
                Some(branch_name_str) => {
                    let branch_name_str_copy = branch_name_str.to_string();
                    delete_local_branch(repo, &mut verified_branch, &branch_name_str_copy)
                }
                None => {}
            },
            Err(e) => {
                println!("Unable to get name of branch: {}", e)
            }
        },
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
            Ok(_) => {}
            Err(e) => {
                println!("Error when deleting local branch: {}", e)
            }
        };
    }
}

fn handle_upstream_branch(repo: &Repository, branch: &(Branch, BranchType)) {
    match branch.0.get().name() {
        Some(branch_ref) => match repo.branch_upstream_remote(branch_ref) {
            Ok(remote) => match remote.as_str() {
                Some(remote_str) => delete_upstream_branch(repo, branch_ref, remote_str),
                None => {}
            },
            Err(_) => {}
        },
        None => {}
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
                Ok(_) => {}
                Err(e) => {
                    println!("Encountered trouble deleting remote branch: {}", e)
                }
            },
            Err(e) => {
                println!("Failed to find remote: {}", e)
            }
        }
    }
}
