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
        debug!("building history: {}...{}", commit1, commit2);

        let result = self.history_short(commit1, commit2, short)?;
        if !result.is_empty() {
            Ok(result)
        } else {
            let mut result = self.history_short(commit2, commit1, short)?;
            result.iter_mut().for_each(|rec| {
                rec.reversed = Some(true);
            });
            Ok(result)
        }
    }

    fn search_oid(&self, commit: &str) -> Result<Oid, Error> {
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
                reversed: None,
            };
            result.push(history);
        }
        Ok(result)
    }
}

fn append_one(workspace: &mut GitWorkspace, package: &mut PackageDiff, short: bool) -> Option<()> {
    if let PackageDiff::Changed {
        first,
        second,
        history,
    } = package
    {
        let v1 = &first.version.as_ref()?;
        let v2 = &second.version.as_ref()?;
        let uri = second.get_git_source().or_else(|| {
            error!("unsupported uri");
            None
        })?;

        let repo = workspace
            .create_repo(&uri)
            .map_err(|_| {
                error!("can't get repo from {}", uri);
            })
            .ok()?;

        let commits = GitHistoryBuilder { repo: &repo }
            .history(v1, v2, short)
            .map_err(|_| {
                error!("can't build detailed history for {}", uri);
            })
            .ok()?;

        debug!("add {} commits to {}", commits.len(), second.name);

        let history = history.get_or_insert(Vec::new());
        commits.iter().for_each(|r| history.push(r.clone()));
        Some(())
    } else {
        None
    }
}

pub fn append(diffs: &mut PackagesDiff, options: &Options) -> Result<(), Error> {
    let mut workspace = GitWorkspace::new(options);
    workspace.init()?;
    for (_, c) in diffs.iter_mut() {
        append_one(&mut workspace, c, options.short_history);
    }
    Ok(())
}
