use base::package::PackageReader;
use base::package::{Package, PackageSource, PackageSources, Packages};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
struct Downloads {
    source: String,
    uris: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ShowInfoPackage {
    name: Option<String>,
    version: Option<String>,
    downloads: Option<Vec<Downloads>>,
}

type ShowInfoPackages = HashMap<String, ShowInfoPackage>;

fn validate_package(package: &ShowInfoPackage) -> Option<&ShowInfoPackage> {
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

fn make_sources(downloads: &Vec<Downloads>) -> PackageSources {
    let mut result = PackageSources::new();
    for download in downloads {
        for uri in &download.uris {
            result.push(PackageSource::from_str(uri).unwrap())
        }
    }
    result
}

fn make_pkgino(input: &ShowInfoPackage) -> Option<Package> {
    validate_package(input).map(|pkg| Package {
        name: pkg.name.as_ref().unwrap().clone(),
        version: pkg.version.clone(),
        sources: make_sources(input.downloads.as_ref().unwrap()),
        location: None,
    })
}

fn convert(input: &ShowInfoPackages) -> Packages {
    let mut output = Packages::new();
    for (k, v) in input {
        make_pkgino(v).map(|x| output.insert(k.clone(), x));
    }

    output
}

pub struct ReportReader {
    path: String,
}

impl ReportReader {
    pub fn new(path: &str) -> ReportReader {
        ReportReader {
            path: path.to_owned(),
        }
    }
}

impl PackageReader for ReportReader {
    type Error = std::io::Error;
    fn read(&mut self) -> Result<Packages, Self::Error> {
        let mut file = File::open(&self.path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;
        let data: ShowInfoPackages = serde_json::from_str(&data)?;
        let res = convert(&data);

        debug!("read {} packages from {}", res.len(), self.path);
        Ok(convert(&data))
    }
}
