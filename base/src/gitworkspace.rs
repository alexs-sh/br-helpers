use git2::*;
use log::{debug, info, warn};
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

fn is_dir_exist(path: &str) -> bool {
    std::fs::read_dir(path).is_ok()
}

#[derive(Clone)]
pub struct Options {
    pub workdir: String,
    pub key: Option<String>,
    pub clean_workspace: bool,
    pub short_history: bool,
}

impl Options {
    pub fn new(workdir: &str) -> Options {
        Options {
            workdir: workdir.to_owned(),
            key: None,
            clean_workspace: false,
            short_history: true,
        }
    }
}

pub struct GitWorkspace {
    options: Options,
}

impl GitWorkspace {
    pub fn new(options: &Options) -> GitWorkspace {
        GitWorkspace {
            options: options.clone(),
        }
    }

    pub fn init(&self) -> Result<(), Error> {
        let path = &self.options.workdir;

        if self.options.clean_workspace && is_dir_exist(path) {
            debug!("removing directory: {}", path);
            std::fs::remove_dir_all(path)?;
        }

        if !is_dir_exist(path) {
            debug!("creating directory: {}", path);
            std::fs::create_dir_all(path)
        } else {
            Ok(())
        }
    }

    pub fn create_repo(&self, uri: &str) -> Result<Repository, Error> {
        let repo = get_repo_name(uri)
            .ok_or_else(|| Error::new(ErrorKind::Other, "can't find repo name"))?;

        let path = format!("{}/{}", self.options.workdir, repo);

        if !is_dir_exist(&path) {
            self.clone_repo(uri, &path)?;
        }

        self.open_repo(&path)
    }

    fn open_repo(&self, path: &str) -> Result<Repository, Error> {
        info!("opening repo {}", path);
        Repository::open(path).map_err(|_| Error::new(ErrorKind::Other, "repo open error"))
    }

    fn clone_repo(&self, uri: &str, path: &str) -> Result<(), Error> {
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
    }
}
