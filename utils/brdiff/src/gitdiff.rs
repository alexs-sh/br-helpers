use crate::pkgdiff::{PkgDiff, PkgDiffs, PkgHistory, PkgHistoryRecord};
use git2::*;
use log::{debug, error, info, warn};
use std::io::{Error, ErrorKind};
use std::path::Path;

fn get_repo_name(uri: &str) -> Option<&str> {
    let parts = uri.split('/');
    let len = uri.len();
    let git = len > 4 && &uri[len - 4..len] == ".git";

    if git && parts.clone().count() > 1 {
        parts.last().map(|s| {
            let len = s.len();
            &s[..len - 4]
        })
    } else {
        None
    }
}

struct RepoHistory<'a> {
    repo: &'a Repository,
}

impl<'a> RepoHistory<'a> {
    pub fn history(&self, commit1: &str, commit2: &str, short: bool) -> Result<PkgHistory, Error> {
        let result = self.history_short(commit1, commit2, short)?;
        if !result.is_empty() {
            Ok(result)
        } else {
            self.history_short(commit2, commit1, short)
        }
    }

    pub fn search_oid(&self, commit: &str) -> Result<Oid, Error> {
        debug!("searching object:{}", commit);

        let parsed = self.repo.revparse_single(commit).map_err(|_| {
            error!("failed to find object: {}", commit);
            Error::new(ErrorKind::Other, "failed to find object")
        })?;

        Ok(parsed.id())
    }

    fn history_short(
        &self,
        commit1: &str,
        commit2: &str,
        short: bool,
    ) -> Result<PkgHistory, Error> {
        let mut result = Vec::new();
        let c1 = self.search_oid(commit1)?;
        let c2 = self.search_oid(commit2)?;

        let mut walk = self.repo.revwalk().unwrap();
        walk.push_range(&format!("{}..{}", c1, c2)).unwrap();

        if short {
            walk.simplify_first_parent().unwrap();
        }

        for commit in walk {
            let id = commit.unwrap();
            let info = self.repo.find_commit(id).unwrap();
            let history = PkgHistoryRecord {
                summary: info.summary().map(|s| s.to_owned()),
                author: Some(info.author().to_string()),
                id: Some(id.to_string()),
            };
            result.push(history);
        }
        Ok(result)
    }
}

#[derive(Clone)]
pub struct HistoryBuilderOptions {
    pub workdir: String,
    pub key: Option<String>,
    pub clean_workdir: bool,
    pub short_history: bool,
}

impl HistoryBuilderOptions {
    pub fn new(workdir: &str) -> HistoryBuilderOptions {
        HistoryBuilderOptions {
            workdir: workdir.to_owned(),
            key: None,
            clean_workdir: false,
            short_history: true,
        }
    }
}

pub struct RepoHistoryBuilder {
    options: HistoryBuilderOptions,
}

impl RepoHistoryBuilder {
    pub fn new(options: &HistoryBuilderOptions) -> RepoHistoryBuilder {
        RepoHistoryBuilder {
            options: options.clone(),
        }
    }

    pub fn init(&self) -> Result<(), Error> {
        let path = &self.options.workdir;
        let exists = std::fs::read_dir(path).is_ok();

        if self.options.clean_workdir && !exists {
            debug!("removing directory: {}", path);
            std::fs::remove_dir_all(path)?;
        }

        let exists = std::fs::read_dir(path).is_ok();
        if !exists {
            debug!("creating directory: {}", path);
            std::fs::create_dir_all(path)
        } else {
            Ok(())
        }
    }

    pub fn history(&self, uri: &str, commit1: &str, commit2: &str) -> Result<PkgHistory, Error> {
        let repo = self.init_repo(uri)?;
        RepoHistory { repo: &repo }.history(commit1, commit2, self.options.short_history)
    }

    fn init_repo(&self, uri: &str) -> Result<Repository, Error> {
        let repo = get_repo_name(uri)
            .ok_or_else(|| Error::new(ErrorKind::Other, "can't find repo name"))?;
        let path = format!("{}/{}", self.options.workdir, repo);
        self.clone_repo(uri, &path)?;
        info!("opening repo {}", path);
        let repo =
            Repository::open(path).map_err(|_| Error::new(ErrorKind::Other, "repo open error"))?;

        Ok(repo)
    }

    fn clone_repo(&self, uri: &str, path: &str) -> Result<(), Error> {
        if std::fs::read_dir(path).is_err() {
            info!("cloning {} into {}", uri, path);
            let mut callbacks = RemoteCallbacks::new();
            let mut builder = git2::build::RepoBuilder::new();
            let mut options = git2::FetchOptions::new();

            if let Some(key) = &self.options.key {
                callbacks.credentials(|_url, username_from_url, _allowed_types| {
                    Cred::ssh_key(
                        username_from_url.unwrap(),
                        None,
                        std::path::Path::new(key),
                        None,
                    )
                });
            }
            options.remote_callbacks(callbacks);
            builder.fetch_options(options);
            builder.clone(uri, Path::new(&path)).map_err(|err| {
                warn!("clone error:{}", err);
                Error::new(ErrorKind::Other, "repo clone error")
            })?;
            Ok(())
        } else {
            info!("skip cloning.{} already exists", path);
            Ok(())
        }
    }
}

pub fn append_history(diffs: &mut PkgDiffs, options: &HistoryBuilderOptions) {
    let repo = RepoHistoryBuilder::new(options);
    repo.init().unwrap();

    for (_, c) in diffs.iter_mut() {
        if let PkgDiff::Changed {
            first,
            second,
            history,
        } = c
        {
            if let Some(uri) = second.get_git_source() {
                match repo.history(&uri, &first.version, &second.version) {
                    Ok(info) => {
                        debug!("add {} commits to {}", info.len(), second.name);
                        let history = history.get_or_insert(PkgHistory::new());
                        info.iter().for_each(|r| history.push(r.clone()));
                    }
                    Err(err) => {
                        warn!("{}", err)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_repo_name() {
        assert_eq!(
            get_repo_name("git@github.com:rust-lang/rust.git"),
            Some("rust")
        );
        assert_eq!(get_repo_name("git@github.com:ghc/ghc.git"), Some("ghc"));
        assert_eq!(
            get_repo_name("https://github.com/torvalds/linux.git"),
            Some("linux")
        );
        assert_eq!(get_repo_name("https://github.com/torvalds/linux.gi"), None);
        assert_eq!(get_repo_name("https:|linux.git"), None);
    }
}
