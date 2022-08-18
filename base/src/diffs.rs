use crate::package::{Package, Packages};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct PackageChange {
    pub summary: Option<String>,
    pub author: Option<String>,
    pub id: Option<String>,
    pub reversed: Option<bool>,
}

pub enum PackageDiff {
    Added {
        package: Package,
    },
    Changed {
        first: Package,
        second: Package,
        history: Option<Vec<PackageChange>>,
    },
    Removed {
        package: Package,
    },
}

impl PackageDiff {
    pub fn name(&self) -> &String {
        match self {
            PackageDiff::Added { package } => &package.name,
            PackageDiff::Changed { second, .. } => &second.name,
            PackageDiff::Removed { package } => &package.name,
        }
    }
}

pub type PackagesDiff = HashMap<String, PackageDiff>;

pub fn build(first: &Packages, second: &Packages) -> PackagesDiff {
    let mut result = PackagesDiff::new();

    //changed + removed
    for (name, package) in first {
        if let Some(info) = second.get(name) {
            if package != info {
                result.insert(
                    name.clone(),
                    PackageDiff::Changed {
                        first: package.clone(),
                        second: info.clone(),
                        history: None,
                    },
                );
            }
        } else {
            result.insert(
                name.clone(),
                PackageDiff::Removed {
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
                PackageDiff::Added {
                    package: package.clone(),
                },
            );
        }
    }

    result
}

impl Display for PackageChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(val) = &self.summary {
            writeln!(f, "       - {}", val)?;
        }
        if let Some(val) = &self.id {
            writeln!(f, "           - id: {}", val)?;
        }
        if let Some(val) = &self.author {
            writeln!(f, "           - author: {}", val)?;
        }
        if let Some(val) = &self.reversed {
            let direction = if *val { "reversed" } else { "direct" };
            writeln!(f, "           - direction: {}", direction)?;
        }
        Ok(())
    }
}

impl Display for PackageDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageDiff::Added { package } => {
                writeln!(f, "[+] {} [added]", package.name)?;
                if let Some(ver) = package.version.as_ref() {
                    writeln!(f, "      version: {}", ver)?;
                }
            }
            PackageDiff::Removed { package } => {
                writeln!(f, "[-] {} [removed]", package.name)?;
            }
            PackageDiff::Changed {
                first,
                second,
                history,
            } => {
                writeln!(f, "[*] {} [modified]", first.name)?;
                let got_versions = first.version.is_some() && second.version.is_some();
                if got_versions && first.version != second.version {
                    writeln!(
                        f,
                        "      version: {} -> {}",
                        first.version.as_ref().unwrap(),
                        second.version.as_ref().unwrap()
                    )?;
                }

                if first.sources != second.sources {
                    writeln!(f, "      sources: changed")?;
                }
                // show history if it presented
                if let Some(hist) = history.as_ref() {
                    hist.iter().for_each(|rec| {
                        let _ = rec.fmt(f);
                    });
                };
            }
        };
        Ok(())
    }
}
