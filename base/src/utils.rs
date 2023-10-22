use std::env;

pub fn get_home_dir() -> Option<String> {
    for v in env::vars() {
        if v.0 == "HOME" {
            return Some(v.1.to_owned());
        }
    }
    None
}
