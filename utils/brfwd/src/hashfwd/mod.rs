use base::gitworkspace;
use base::{gitworkspace::GitWorkspace, package::Package};

use log::info;
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

pub fn get_tag(ws: &mut GitWorkspace, url: &str, tag: &str, abbrev: u32) -> Option<String> {
    let repo = ws.create_repo(url).unwrap();
    gitworkspace::get_tag(&repo, tag).map(|object| {
        if abbrev > 0 {
            gitworkspace::get_object_abbrev(&object, Some(abbrev))
        } else {
            gitworkspace::get_object_hash(&object)
        }
    })
}

pub fn get_latest_commit(
    ws: &mut GitWorkspace,
    url: &str,
    head: &str,
    abbrev: u32,
) -> Option<String> {
    let repo = ws.create_repo(url).unwrap();
    gitworkspace::get_latest_commit(&repo, head).map(|object| {
        if abbrev > 0 {
            gitworkspace::get_object_abbrev(&object, Some(abbrev))
        } else {
            gitworkspace::get_object_hash(&object)
        }
    })
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
