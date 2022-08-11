use crate::diffs::PackagesDiff;

pub fn print_diffs(diffs: &PackagesDiff) {
    diffs.iter().for_each(|(_, diff)| println!("{}", diff));
}
