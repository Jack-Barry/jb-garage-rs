use config::get_config_from_dir;
use fs_tools::use_current_working_dir;

mod config;
mod fs_tools;

fn main() {
    match use_current_working_dir(get_config_from_dir) {
        Ok(_) => {
            println!("Success! 🥳 Thank you for using Versioner")
        }
        Err(e) => {
            println!("{:?}", e)
        }
    }
}
