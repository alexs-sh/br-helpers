use crate::pkginfo::{PkgInfo, PkgInfos};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct PkgHistoryRecord {
    pub summary: Option<String>,
    pub author: Option<String>,
    pub id: Option<String>,
}

pub enum PkgDiff {
    Added {
        package: PkgInfo,
    },
    Changed {
        first: PkgInfo,
        second: PkgInfo,
        history: Option<PkgHistory>,
    },
    Removed {
        package: PkgInfo,
    },
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
pub type PkgHistory = Vec<PkgHistoryRecord>;

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
                        history: None,
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

pub fn print_diffs(diffs: &PkgDiffs) {
    diffs.iter().for_each(|(_, diff)| println!("{}", diff));
}

impl Display for PkgHistoryRecord {
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
        Ok(())
    }
}

impl Display for PkgDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PkgDiff::Added { package } => {
                writeln!(f, "[+] {} [added]", package.name)?;
                writeln!(f, "      version: {}", package.version)?;
            }
            PkgDiff::Removed { package } => {
                writeln!(f, "[-] {} [removed]", package.name)?;
            }
            PkgDiff::Changed {
                first,
                second,
                history,
            } => {
                writeln!(f, "[*] {} [modified]", first.name)?;
                if first.version != second.version {
                    writeln!(f, "      version: {} -> {}", first.version, second.version)?;
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

                //for s in &first.sources {
                //    println!("    - sources A: {:?}",s);
                //}
                //for s in &second.sources {
                //    println!("    - sources B: {:?}",s);
                //}}
            }
        };
        Ok(())
    }
}
