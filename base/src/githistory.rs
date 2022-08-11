use crate::diffs::{PackageChange, PackageDiff, PackagesDiff};
use crate::gitworkspace::{GitWorkspace, Options};

use git2::*;
use log::{debug, error};
use std::io::{Error, ErrorKind};

struct GitHistoryBuilder<'a> {
    repo: &'a Repository,
}

impl<'a> GitHistoryBuilder<'a> {
    fn history(
        &self,
        commit1: &str,
        commit2: &str,
        short: bool,
    ) -> Result<Vec<PackageChange>, Error> {
        let result = self.history_short(commit1, commit2, short)?;
        if !result.is_empty() {
            Ok(result)
        } else {
            self.history_short(commit2, commit1, short)
        }
    }

    fn search_oid(&self, commit: &str) -> Result<Oid, Error> {
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
    ) -> Result<Vec<PackageChange>, Error> {
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
            let history = PackageChange {
                summary: info.summary().map(|s| s.to_owned()),
                author: Some(info.author().to_string()),
                id: Some(id.to_string()),
            };
            result.push(history);
        }
        Ok(result)
    }
}

pub fn append(diffs: &mut PackagesDiff, options: &Options) -> Result<(), Error> {
    let workspace = GitWorkspace::new(options);
    workspace.init()?;

    for (_, c) in diffs.iter_mut() {
        if let PackageDiff::Changed {
            first,
            second,
            history,
        } = c
        {
            if let Some(uri) = second.get_git_source() {
                let repo = workspace.create_repo(&uri)?;
                let commits = GitHistoryBuilder { repo: &repo }.history(
                    &first.version,
                    &second.version,
                    options.short_history,
                )?;

                debug!("add {} commits to {}", commits.len(), second.name);

                let history = history.get_or_insert(Vec::new());
                commits.iter().for_each(|r| history.push(r.clone()));
            }
        }
    }
    Ok(())
}
