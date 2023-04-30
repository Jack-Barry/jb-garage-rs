use jbg_fs::use_current_working_dir;
use versioner_core::get_config_from_dir;

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
