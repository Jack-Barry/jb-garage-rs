use std::{
    env,
    error::Error,
    path::{self, PathBuf},
};

#[derive(Debug)]
pub enum JbgFsError {
    CwdError(Box<dyn Error>),
}

/// Determines if entity exists at provided path
pub fn file_or_dir_exists(path: &str) -> bool {
    let entity = path::Path::new(path);
    entity.exists()
}

/// Invokes provided callback if current working dir is available
pub fn use_current_working_dir<F: Fn(PathBuf) -> Result<(), Box<dyn std::error::Error>>>(
    handle_working_dir: F,
) -> Result<(), JbgFsError> {
    let working_dir = env::current_dir();
    match working_dir {
        Ok(cwd) => match handle_working_dir(cwd) {
            Ok(_) => Ok(()),
            Err(e) => Err(JbgFsError::CwdError(e)),
        },
        Err(e) => Err(JbgFsError::CwdError(Box::new(e))),
    }
}
