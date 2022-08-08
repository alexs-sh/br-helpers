use crate::pkginfo::{PkgInfo, PkgInfos, PkgSource, PkgSources};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::path::Path;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
struct Downloads {
    source: String,
    uris: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Package {
    name: Option<String>,
    version: Option<String>,
    downloads: Option<Vec<Downloads>>,
}

type Packages = HashMap<String, Package>;

fn validate_package(package: &Package) -> Option<&Package> {
    if package.name.is_some()
        && package.version.is_some()
        && package.downloads.is_some()
        && !package.downloads.as_ref().unwrap().is_empty()
    {
        Some(package)
    } else {
        None
    }
}

fn make_sources(downloads: &Vec<Downloads>) -> PkgSources {
    let mut result = PkgSources::new();
    for download in downloads {
        for uri in &download.uris {
            result.push(PkgSource::from_str(uri).unwrap())
        }
    }
    result
}

fn make_pkgino(input: &Package) -> Option<PkgInfo> {
    validate_package(input).map(|pkg| PkgInfo {
        name: pkg.name.as_ref().unwrap().clone(),
        version: pkg.version.as_ref().unwrap().clone(),
        sources: make_sources(input.downloads.as_ref().unwrap()),
    })
}

fn convert(input: &Packages) -> PkgInfos {
    let mut output = PkgInfos::new();
    for (k, v) in input {
        make_pkgino(v).map(|x| output.insert(k.clone(), x));
    }

    output
}

pub fn read(path: &Path) -> Result<PkgInfos, Error> {
    let mut file = File::open(path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let data: Packages = serde_json::from_str(&data)?;
    Ok(convert(&data))
}
