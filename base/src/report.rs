use crate::diffs::PackagesDiff;
use std::io::{Error, Write};

pub fn print_diffs(diffs: &PackagesDiff) {
    diffs.iter().for_each(|(_, diff)| println!("{}", diff));
}

pub fn write_diffs(file: &str, diffs: &PackagesDiff) -> Result<(), Error> {
    let mut file = std::fs::File::create(file).or_else(|_| std::fs::File::open(file))?;

    diffs.iter().for_each(|(_, diff)| {
        let _ = file.write_fmt(format_args!("{}", diff));
    });
    Ok(())
}
