use crate::pkgdiff::{self, PkgDiff, PkgDiffs};
use git2::*;
use log::{debug, warn};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::path::Path;
use uuid::Uuid;

fn create_repo(workdir: &str, uri: &str, key: Option<String>) -> Result<Repository, Error> {
    let id = Uuid::new_v4();
    let dst = format!("{}/{}", workdir, id);
    debug!("cloning {} to {}", uri, dst);

    let mut callbacks = RemoteCallbacks::new();

    let mut builder = git2::build::RepoBuilder::new();
    let mut options = git2::FetchOptions::new();

    if key.is_some() {
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key(
                username_from_url.unwrap(),
                None,
                std::path::Path::new(key.as_ref().unwrap_or(&"~/.ssh/id_rsa".to_string())),
                None,
            )
        });
    }

    options.remote_callbacks(callbacks);
    builder.fetch_options(options);
    builder.clone(uri, Path::new(&dst)).map_err(|err| {
        warn!("clone error:{}", err);
        Error::new(ErrorKind::Other, "repo clone error")
    })?;

    let repo =
        Repository::open(dst).map_err(|_| Error::new(ErrorKind::Other, "repo open error"))?;

    Ok(repo)
}

fn search_commit<'a>(repo: &'a Repository, commit: &str) -> Result<Commit<'a>, Error> {
    debug!("searching commit {} ", commit);
    let oid = Oid::from_str(commit).map_err(|_| Error::new(ErrorKind::Other, "oid error"))?;

    let commit = repo
        .find_commit(oid)
        .map_err(|_| Error::new(ErrorKind::Other, "commit error"))?;
    Ok(commit)
}

fn sort_commit<'a>(c1: Commit<'a>, c2: Commit<'a>) -> (Commit<'a>, Commit<'a>) {
    let c1_time = c1.time().seconds() - (c1.time().offset_minutes() as i64) * 60;
    let c2_time = c2.time().seconds() - (c2.time().offset_minutes() as i64) * 60;
    if c1_time <= c2_time {
        (c1, c2)
    } else {
        (c2, c1)
    }
}

fn walk(
    repo: &Repository,
    commit1: &str,
    commit2: &str,
    short: bool,
) -> Result<Vec<String>, Error> {
    debug!("walking at {:?}", repo.workdir());

    let mut result = Vec::new();
    let c1 = search_commit(repo, commit1)?;
    let c2 = search_commit(repo, commit2)?;
    let (c1, c2) = sort_commit(c1, c2);

    let mut walk = repo.revwalk().unwrap();
    walk.push_range(&format!("{}..{}", c1.id(), c2.id()))
        .unwrap();

    if short {
        walk.simplify_first_parent().unwrap();
    }

    for commit in walk {
        let id = commit.unwrap();
        let info = repo.find_commit(id).unwrap();
        result.push(info.summary().unwrap_or("(NO SUMMARY)").to_owned());
    }
    Ok(result)
}

pub fn history(
    workdir: &str,
    uri: &str,
    commit1: &str,
    commit2: &str,
    short: bool,
    key: Option<String>,
) -> Result<Vec<String>, Error> {
    let repo = create_repo(workdir, uri, key)?;
    walk(&repo, commit1, commit2, short)
}

pub fn print_diffs(workdir: &str, diffs: &PkgDiffs, short: bool, key: Option<String>) {
    let mut commits = HashMap::new();
    for c in diffs.values() {
        if let PkgDiff::Changed { first, second } = c {
            if let Some(uri) = second.get_git_source() {
                match history(
                    workdir,
                    &uri,
                    &first.version,
                    &second.version,
                    short,
                    key.clone(),
                ) {
                    Ok(info) => {
                        debug!("add {} commits to {}", info.len(), second.name);
                        commits.insert(second.name.clone(), info);
                    }
                    Err(err) => {
                        warn!("{}", err)
                    }
                }
            }
        }
    }

    for d in diffs.values() {
        pkgdiff::print_diff(d);
        if let Some(commits) = commits.get(d.name()) {
            for c in commits {
                println!("        - {}", c);
            }
        }
    }
}
