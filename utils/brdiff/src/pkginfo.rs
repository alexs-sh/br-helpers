use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum PkgSource {
    Git(String),
    Https(String),
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PkgInfo {
    pub name: String,
    pub version: String,
    pub sources: PkgSources,
}

impl PkgInfo {
    pub fn get_git_source(&self) -> Option<String> {
        for s in &self.sources {
            if let PkgSource::Git(uri) = s {
                return Some(uri.to_owned());
            }
        }
        None
    }
}

pub type PkgInfos = HashMap<String, PkgInfo>;
pub type PkgSources = Vec<PkgSource>;

impl FromStr for PkgSource {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let prefixes = ["git+", "https+", "other"];
        let mode = prefixes.iter().find(|&x| s.contains(x));
        let output = match mode {
            Some(&"https+") => PkgSource::Https(s[6..].to_owned()),
            Some(&"git+") => PkgSource::Git(s[4..].to_owned()),
            _ => PkgSource::Other(s.to_owned()),
        };
        Ok(output)
    }
}

#[cfg(test)]

mod test {
    use super::*;
    #[test]
    fn from_str_https() {
        match PkgSource::from_str("https+https://snapshot.debian.org/archive/debian/20201008T205817Z/pool/main/f/fakeroot").unwrap()
        {
            PkgSource::Https(src) => {
                assert_eq!(src,"https://snapshot.debian.org/archive/debian/20201008T205817Z/pool/main/f/fakeroot")
            },
            _ => {unimplemented!()}
        }
    }

    #[test]
    fn from_str_git() {
        match PkgSource::from_str("git+git@github.com:rust-lang/rust.git").unwrap() {
            PkgSource::Git(src) => {
                assert_eq!(src, "git@github.com:rust-lang/rust.git")
            }
            _ => {
                unimplemented!()
            }
        }
    }

    #[test]
    fn from_str_other() {
        match PkgSource::from_str("local+/tmp/build/custom/super-package").unwrap() {
            PkgSource::Other(src) => {
                assert_eq!(src, "local+/tmp/build/custom/super-package")
            }
            _ => {
                unimplemented!()
            }
        }
    }
}
