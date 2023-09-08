mod hashfwd;

use base::{
    gitworkspace::{GitWorkspace, Options as GitWorkspaceOptions},
    mkfile,
    package::PackageReader,
};

use log::{error, info, warn};
use std::collections::HashSet;
use std::io::Error;
use structopt::StructOpt;
#[derive(Debug, StructOpt)]
#[structopt(
    name = "brfwd",
    about = "this tool changes package versions in BR mk-files using data\nfrom a package repository and user-specified parameters,\nlike branch or tag name, version format, and so on."
)]
struct Options {
    #[structopt(
        short = "i",
        long = "input",
        default_value = "package.mk",
        help = "path to the mk file or directory with mk files"
    )]
    input: String,

    #[structopt(
        short = "w",
        long = "workdir",
        default_value = "/tmp/brfwd",
        help = "path to the working directory"
    )]
    workdir: String,

    #[structopt(short = "k", long = "key", help = "SSH key", default_value = "")]
    key: String,

    #[structopt(
        short = "c",
        long = "clean",
        parse(try_from_str),
        default_value = "false",
        help = "clean working directory before run"
    )]
    clean: bool,

    #[structopt(
        short = "b",
        long = "branch",
        default_value = "origin/master",
        help = "branch name"
    )]
    branch: String,

    #[structopt(
        short = "t",
        long = "tag",
        default_value = "",
        help = "use tag (if exists)"
    )]
    tag: String,

    #[structopt(
        short = "a",
        long = "abbrev",
        parse(try_from_str),
        default_value = "0",
        help = "if nonzero, then the abbreviation of length N will be used as version"
    )]
    abbrev: u32,

    #[structopt(short = "s", long = "skip", default_value = "", help = "skip packages")]
    skip: String,

    #[structopt(
        short = "l",
        long = "limit",
        parse(try_from_str),
        default_value = "0",
        help = "process not more packages than limit. 0 - no limit"
    )]
    limit: usize,
}

fn guess_reader(filename: &str) -> Result<Box<dyn PackageReader<Error = Error>>, Error> {
    if filename.ends_with(".mk") {
        info!("use MkFile reader for {}", filename);
        Ok(Box::new(mkfile::MkFileReader::new(filename)))
    } else if std::fs::read_dir(filename).is_ok() {
        info!("use default dir. reader for {}", filename);
        Ok(Box::new(mkfile::MkFileDirReader::new(filename)))
    } else {
        info!("use default file reader for {}", filename);
        Ok(Box::new(mkfile::MkFileReader::new(filename)))
    }
}

fn blacklist_from_str(input: &str) -> HashSet<String> {
    input.split(',').fold(HashSet::new(), |mut acc, value| {
        info!("add {} into blacklist", value);
        acc.insert(value.to_owned());
        acc
    })
}

fn get_new_version(ws: &mut GitWorkspace, url: &str, opts: &Options) -> Option<String> {
    let mut result = None;
    let mut msg = String::new();
    if !opts.tag.is_empty() {
        msg = format!("{}: switching to {}", url, opts.tag);
        result = hashfwd::get_tag(ws, url, &opts.tag, opts.abbrev);
    } else if !opts.branch.is_empty() {
        msg = format!("{}: switching to the last commit on {}", url, opts.branch);
        result = hashfwd::get_latest_commit(ws, url, &opts.branch, opts.abbrev);
    };

    if result.is_some() {
        info!("{} done", msg);
    } else {
        warn!("{} failed", msg);
    }
    result
}

fn run(opts: Options) -> Result<(), Error> {
    let packages = guess_reader(&opts.input)?.read()?;
    let mut wsopts = GitWorkspaceOptions::new(&opts.workdir);
    let blacklist = blacklist_from_str(&opts.skip);

    wsopts.key = opts.key.clone();
    wsopts.clean_workspace = opts.clean;

    let mut wsgit = GitWorkspace::new(&wsopts);
    wsgit.init()?;

    let limit = if opts.limit == 0 {
        packages.len()
    } else {
        opts.limit
    };

    let mut processed = 0;
    for (_, package) in packages.iter() {
        info!("{} processing", package.name);
        if !hashfwd::check_package(package, &blacklist) {
            continue;
        }

        let url = package.get_git_source().unwrap();
        if let Some(hash) = get_new_version(&mut wsgit, &url, &opts) {
            info!("use {} for {}", hash, url);
            hashfwd::replace_commit(
                package.location.as_ref().unwrap(),
                package.version.as_ref().unwrap(),
                &hash,
            )
            .unwrap();

            processed += 1;
            if processed >= limit {
                info!("got limit of processed packages");
                break;
            }
        }
    }
    Ok(())
}

fn main() {
    env_logger::init();
    let opts = Options::from_args();
    match run(opts) {
        Ok(_) => {
            println!("Done")
        }
        Err(err) => {
            error!("brfwd fails:{:?}", err)
        }
    }
}
