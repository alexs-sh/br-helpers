use base::{gitworkspace::GitWorkspace, package::Package};

use log::{debug, info};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::Error;
use std::io::{Read, Write};

pub fn check_package(package: &Package, blacklist: &HashSet<String>) -> bool {
    let name = &package.name;
    if blacklist.contains(name) {
        info!("{} package in blacklist", name);
        return false;
    }

    if package.get_git_source().is_none() {
        info!("{} not a git package", name);
        return false;
    }
    true
}

pub fn get_latest_commit(ws: &mut GitWorkspace, url: &str, head: &str) -> Option<String> {
    let repo = ws.create_repo(url).unwrap();
    for branch in repo.branches(None).unwrap() {
        let info = branch.unwrap().0;
        let name = info.name().unwrap().unwrap();
        debug!("visit branch {}", name);
        if name == head {
            if let Ok(comm) = info.get().peel_to_commit() {
                let id = comm.as_object().id().to_string();
                info!("{} is the latest commit for {}", id, head);
                return Some(id);
            }
        }
    }

    info!("branch {} not found", head);
    None
}

pub fn replace_commit(file: &str, old: &str, new: &str) -> Result<(), Error> {
    let mut input = String::new();
    File::open(file)?.read_to_string(&mut input)?;

    let data = input.replace(old, new);
    let tmpfile = format!("{}.tmp", file);
    let mut out = OpenOptions::new().write(true).create(true).open(&tmpfile)?;

    out.write_all(data.as_bytes())?;
    out.sync_all()?;

    std::fs::rename(&tmpfile, file)?;
    Ok(())
}
