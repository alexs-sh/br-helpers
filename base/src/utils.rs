use std::env;
use std::path::PathBuf;

pub fn get_home_dir() -> Option<String> {
    for v in env::vars() {
        if v.0 == "HOME" {
            return Some(v.1.to_owned());
        }
    }
    None
}

pub fn get_default_ssh_key() -> Option<String> {
    if let Some(home) = get_home_dir() {
        let path: PathBuf = [&home, ".ssh", "id_rsa"].iter().collect();
        return Some(path.to_str().unwrap().to_owned());
    }
    None
}
