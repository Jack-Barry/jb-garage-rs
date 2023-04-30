use std::{
    env,
    path::{self, PathBuf},
};

#[derive(Debug)]
pub enum FSToolsError {
    CwdError,
}

/// Determines if entity exists at provided path
pub fn file_or_dir_exists(path: &str) -> bool {
    let entity = path::Path::new(path);
    entity.exists()
}

/// Invokes provided callback if current working dir is available
pub fn use_current_working_dir<F: Fn(PathBuf)>(handle_working_dir: F) -> Result<(), FSToolsError> {
    let working_dir = env::current_dir();
    match working_dir {
        Ok(cwd) => {
            handle_working_dir(cwd);
            Ok(())
        }
        Err(_) => Err(FSToolsError::CwdError),
    }
}
