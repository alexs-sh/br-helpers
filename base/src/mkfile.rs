use crate::package::{Package, PackageReader, PackageSource, Packages};
use log::{info, warn};
use std::io::{BufRead, BufReader};
use std::io::{Error, ErrorKind};

use walkdir::WalkDir;

pub struct MkFile {
    path: String,
}

fn try_extract_pkgname(path: &str) -> Option<&str> {
    let path = std::path::Path::new(path.trim());
    path.file_name()
        .and_then(|x| x.to_str().and_then(|x| x.strip_suffix(".mk")))
}

fn try_read_pkgname(key: &str) -> Option<String> {
    key.trim().strip_suffix("_VERSION").map(|prefix| {
        let mut result = prefix.to_owned();
        result = result.to_lowercase();
        result = result.replace('_', "-");
        result
    })
}

fn try_read_pkgver<'a>((key, value): (&'a str, &'a str)) -> Option<&'a str> {
    key.trim().strip_suffix("_VERSION").map(|_| value)
}

fn try_read_git_source<'a>((key, value): (&'a str, &'a str)) -> Option<&'a str> {
    let key = key.trim();
    let value = value.trim();
    if key.ends_with("_SITE") && value.ends_with(".git") {
        Some(value)
    } else {
        None
    }
}

fn is_commented(line: &str) -> bool {
    let txt = line.trim();
    match txt.chars().take(1).next() {
        None => false,
        Some(c) => c == '#',
    }
}

fn try_read_cmdline(line: &str) -> Option<&str> {
    let from = line
        .chars()
        .position(|symb| !symb.is_whitespace())
        .unwrap_or(line.len());
    let end = line
        .chars()
        .position(|symb| symb == '#')
        .unwrap_or(line.len());

    if from < end {
        Some(&line[from..end])
    } else {
        None
    }
}

fn try_read_key_value(line: &str) -> Option<(&str, &str)> {
    if let Some(key_idx) = line.chars().position(|symb| symb == '=') {
        if key_idx + 1 < line.len() {
            let key = (&line[..key_idx]).trim();
            let value = (&line[key_idx + 1..]).trim();
            return Some((key, value));
        }
    }
    None
}

impl MkFile {
    pub fn new(path: &str) -> MkFile {
        MkFile {
            path: path.to_owned(),
        }
    }

    pub fn read_info(&self) -> Result<Package, Error> {
        let file = std::fs::File::open(&self.path)?;
        let buf = BufReader::new(file);
        let name_file = try_extract_pkgname(&self.path)
            .ok_or_else(|| Error::new(ErrorKind::Other, "invalid name"))?;
        let mut name_mk = None;
        let mut ver_mk = None;
        let mut source_mk = None;

        for line in buf.lines().flatten() {
            if let Some((k, v)) = try_read_cmdline(&line).and_then(try_read_key_value) {
                if name_mk.is_none() {
                    name_mk = try_read_pkgname(k);
                }

                if ver_mk.is_none() {
                    ver_mk = try_read_pkgver((k, v)).map(|x| x.to_owned());
                }

                if source_mk.is_none() {
                    source_mk = try_read_git_source((k, v)).map(|x| x.to_owned());
                }
            }
        }

        if let Some(name) = name_mk {
            if name != name_file {
                warn!(
                    "different pkg names from filename/mk:{} vs {}",
                    name, name_file
                );
            }
        }

        let sources = source_mk.map_or(Vec::new(), |url| vec![PackageSource::Git(url)]);

        Ok(Package {
            name: name_file.to_owned(),
            version: ver_mk,
            sources,
            location: Some(self.path.to_owned()),
        })
    }
}

pub struct MkFileReader {
    path: String,
}

impl MkFileReader {
    pub fn new(path: &str) -> MkFileReader {
        MkFileReader {
            path: path.to_owned(),
        }
    }
}

impl PackageReader for MkFileReader {
    type Error = Error;
    fn read(&mut self) -> Result<Packages, Self::Error> {
        let mkreader = MkFile::new(&self.path);
        let pkg = mkreader.read_info()?;
        let mut result = Packages::new();
        result.insert(pkg.name.clone(), pkg);
        Ok(result)
    }
}

pub struct MkFileDirReader {
    path: String,
}

impl MkFileDirReader {
    pub fn new(path: &str) -> MkFileDirReader {
        MkFileDirReader {
            path: path.to_owned(),
        }
    }
}

impl PackageReader for MkFileDirReader {
    type Error = Error;
    fn read(&mut self) -> Result<Packages, Self::Error> {
        let mut result = Packages::new();
        let mut last_error = None;
        let entries = WalkDir::new(&self.path);
        for e in entries.into_iter().flatten() {
            if let Some(file) = e.path().to_str() {
                if file.ends_with(".mk") {
                    info!("process {}", file);
                    let mkreader = MkFile::new(file);
                    match mkreader.read_info() {
                        Ok(pkg) => {
                            result.insert(pkg.name.clone(), pkg);
                        }
                        Err(err) => {
                            warn!("failed to process {}:{:?}", file, err);
                            last_error = Some(err);
                        }
                    };
                }
            }
        }

        if result.is_empty() && last_error.is_some() {
            Err(Error::new(ErrorKind::Other, "failed to read mk-files"))
        } else {
            Ok(result)
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn test_is_commented() {
        assert_eq!(is_commented("#simple text"), true);
        assert_eq!(is_commented(" #simple text"), true);
        assert_eq!(is_commented("	#simple text"), true);
        assert_eq!(is_commented("#"), true);
        assert_eq!(is_commented("#simple text"), true);
    }

    #[test]
    fn test_not_commented() {
        assert_eq!(is_commented("simple text#"), false);
        assert_eq!(is_commented("simple#text#"), false);
        assert_eq!(is_commented(""), false);
    }

    #[test]
    fn test_get_cmd() {
        assert_eq!(try_read_cmdline("#commented"), None);
        assert_eq!(try_read_cmdline("text #commented"), Some("text "));
        assert_eq!(try_read_cmdline(""), None);
        assert_eq!(try_read_cmdline("#"), None);
        assert_eq!(
            try_read_cmdline("ololo text bla-bla #comment"),
            Some("ololo text bla-bla ")
        );
    }

    #[test]
    fn test_kv() {
        let (k, v) = try_read_key_value("BOOST_FLAGS = --with-toolset=gcc").unwrap();
        assert_eq!(k, "BOOST_FLAGS");
        assert_eq!(v, "--with-toolset=gcc");

        let (k, v) = try_read_key_value("  BOOST_FLAGS   =    --with-toolset=gcc   ").unwrap();
        assert_eq!(k, "BOOST_FLAGS");
        assert_eq!(v, "--with-toolset=gcc");

        let (k, v) = try_read_key_value("BOOST_FLAGS=--with-toolset=gcc").unwrap();
        assert_eq!(k, "BOOST_FLAGS");
        assert_eq!(v, "--with-toolset=gcc");
    }

    #[test]
    fn test_not_kv() {
        let res = try_read_key_value("BOOST_FLAGS =");
        assert_eq!(res, None);

        let res = try_read_key_value("BOOST_FLAGS");
        assert_eq!(res, None);
    }

    #[test]
    fn test_pkg_from_file() {
        let res = try_extract_pkgname("boost.mk").unwrap();
        assert_eq!(res, "boost");

        let res = try_extract_pkgname("/test/ololo/boost.mk").unwrap();
        assert_eq!(res, "boost");

        let res = try_extract_pkgname("/test/ololo/super-duper-pack.mk   ").unwrap();
        assert_eq!(res, "super-duper-pack");
    }

    #[test]
    fn test_pkg_from_mkline() {
        let res = try_read_pkgname("BOOST_VERSION").unwrap();
        assert_eq!(res, "boost");

        let res = try_read_pkgname("  BOOST_VERSION  ").unwrap();
        assert_eq!(res, "boost");

        let res = try_read_pkgname("MAGIC_PACKAGE_VERSION").unwrap();
        assert_eq!(res, "magic-package");
    }

    #[test]
    fn test_pkg_git_source() {
        let kv = try_read_key_value("MAGIC_PACKAGE_SITE = git@ololo.git").unwrap();
        let res = try_read_git_source(kv).unwrap();
        assert_eq!(res, "git@ololo.git");

        let kv = try_read_key_value("   MAGIC_PACKAGE_SITE = git@ololo.git  ").unwrap();
        let res = try_read_git_source(kv).unwrap();
        assert_eq!(res, "git@ololo.git");
    }
}
