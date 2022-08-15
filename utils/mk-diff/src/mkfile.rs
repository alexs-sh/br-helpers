use base::package::Package;
use std::io::{BufRead, BufReader};
use std::io::{Error, ErrorKind};

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

fn try_as_key_value(line: &str) -> Option<(&str, &str)> {
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
        let pkg_name = try_extract_pkgname(&self.path)
            .ok_or_else(|| Error::new(ErrorKind::Other, "invalid name"))?;
        let mut pkg_name_mk = None;
        let mut pkg_ver_mk = None;

        for line in buf.lines().flatten() {
            if let Some((k, v)) = try_read_cmdline(&line).and_then(try_as_key_value) {
                if pkg_name_mk.is_none() {
                    pkg_name_mk = try_read_pkgname(k);
                }

                if pkg_ver_mk.is_none() {
                    pkg_ver_mk = try_read_pkgver((k, v)).map(|x| x.to_owned());
                }
            }
        }

        if let Some(name) = pkg_name_mk {
            assert_eq!(name, pkg_name);
        }

        Ok(Package {
            name: pkg_name.to_owned(),
            version: pkg_ver_mk.unwrap(),
            sources: Vec::new(),
        })
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
        let (k, v) = try_as_key_value("BOOST_FLAGS = --with-toolset=gcc").unwrap();
        assert_eq!(k, "BOOST_FLAGS");
        assert_eq!(v, "--with-toolset=gcc");

        let (k, v) = try_as_key_value("  BOOST_FLAGS   =    --with-toolset=gcc   ").unwrap();
        assert_eq!(k, "BOOST_FLAGS");
        assert_eq!(v, "--with-toolset=gcc");

        let (k, v) = try_as_key_value("BOOST_FLAGS=--with-toolset=gcc").unwrap();
        assert_eq!(k, "BOOST_FLAGS");
        assert_eq!(v, "--with-toolset=gcc");
    }

    #[test]
    fn test_not_kv() {
        let res = try_as_key_value("BOOST_FLAGS =");
        assert_eq!(res, None);

        let res = try_as_key_value("BOOST_FLAGS");
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
}
