use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum PackageSource {
    Git(String),
    Https(String),
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub sources: PackageSources,
}

impl Package {
    pub fn get_git_source(&self) -> Option<String> {
        for s in &self.sources {
            if let PackageSource::Git(uri) = s {
                return Some(uri.to_owned());
            }
        }
        None
    }
}

pub type Packages = HashMap<String, Package>;
pub type PackageSources = Vec<PackageSource>;

impl FromStr for PackageSource {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let prefixes = ["git+", "https+", "other"];
        let mode = prefixes.iter().find(|&x| s.contains(x));
        let output = match mode {
            Some(&"https+") => PackageSource::Https(s[6..].to_owned()),
            Some(&"git+") => PackageSource::Git(s[4..].to_owned()),
            _ => PackageSource::Other(s.to_owned()),
        };
        Ok(output)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn from_str_https() {
        match PackageSource::from_str("https+https://snapshot.debian.org/archive/debian/20201008T205817Z/pool/main/f/fakeroot").unwrap()
        {
            PackageSource::Https(src) => {
                assert_eq!(src,"https://snapshot.debian.org/archive/debian/20201008T205817Z/pool/main/f/fakeroot")
            },
            _ => {unimplemented!()}
        }
    }

    #[test]
    fn from_str_git() {
        match PackageSource::from_str("git+git@github.com:rust-lang/rust.git").unwrap() {
            PackageSource::Git(src) => {
                assert_eq!(src, "git@github.com:rust-lang/rust.git")
            }
            _ => {
                unimplemented!()
            }
        }
    }

    #[test]
    fn from_str_other() {
        match PackageSource::from_str("local+/tmp/build/custom/super-package").unwrap() {
            PackageSource::Other(src) => {
                assert_eq!(src, "local+/tmp/build/custom/super-package")
            }
            _ => {
                unimplemented!()
            }
        }
    }
}
