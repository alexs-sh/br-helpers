use crate::pkginfo::{PkgInfo, PkgInfos};
use std::collections::HashMap;

pub enum PkgDiff {
    Added { package: PkgInfo },
    Changed { first: PkgInfo, second: PkgInfo },
    Removed { package: PkgInfo },
}

impl PkgDiff {
    pub fn name(&self) -> &String {
        match self {
            PkgDiff::Added { package } => &package.name,
            PkgDiff::Changed { second, .. } => &second.name,
            PkgDiff::Removed { package } => &package.name,
        }
    }
}

pub type PkgDiffs = HashMap<String, PkgDiff>;

pub fn build(first: &PkgInfos, second: &PkgInfos) -> PkgDiffs {
    let mut result = PkgDiffs::new();

    //changed + removed
    for (name, package) in first {
        if let Some(info) = second.get(name) {
            if package != info {
                result.insert(
                    name.clone(),
                    PkgDiff::Changed {
                        first: package.clone(),
                        second: info.clone(),
                    },
                );
            }
        } else {
            result.insert(
                name.clone(),
                PkgDiff::Removed {
                    package: package.clone(),
                },
            );
        }
    }

    // added
    for (name, package) in second {
        if !first.contains_key(name) {
            result.insert(
                name.clone(),
                PkgDiff::Added {
                    package: package.clone(),
                },
            );
        }
    }

    result
}

pub fn print_diff(diff: &PkgDiff) {
    match diff {
        PkgDiff::Added { package } => {
            println!("[+] {} [added]", package.name);
            println!("      version: {}", package.version);
        }
        PkgDiff::Removed { package } => {
            println!("[-] {} [removed]", package.name);
        }
        PkgDiff::Changed { first, second } => {
            println!("[*] {} [modified]", first.name);
            if first.version != second.version {
                println!("      version: {} -> {}", first.version, second.version);
            }
            if first.sources != second.sources {
                println!("      sources: changed");
            }
            //for s in &first.sources {
            //    println!("    - sources A: {:?}",s);
            //}
            //for s in &second.sources {
            //    println!("    - sources B: {:?}",s);
            //}}
        }
    }
}

pub fn print_diffs(diffs: &PkgDiffs) {
    diffs.iter().for_each(|(_, diff)| print_diff(diff));
}
