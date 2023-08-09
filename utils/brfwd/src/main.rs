mod hashfwd;

use base::{
    gitworkspace::{GitWorkspace, Options as GitWorkspaceOptions},
    mkfile,
    package::PackageReader,
};

use log::{debug, error, info, warn};
use std::collections::HashSet;
use std::io::Error;
use structopt::StructOpt;
#[derive(Debug, StructOpt)]
#[structopt(
    name = "brdiff",
    about = "compare two BR package set. The app uses output generated by the 'show-info' command from Buildroot"
)]
struct Options {
    #[structopt(
        short = "i",
        long = "input",
        default_value = "first.json",
        help = "path to the mk(s)"
    )]
    input: String,

    #[structopt(
        short = "w",
        long = "workdir",
        default_value = "/tmp/brfwd",
        help = "path to the working directory"
    )]
    workdir: String,

    #[structopt(short = "k", long = "key", help = "SSH key")]
    key: Option<String>,

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

    #[structopt(short = "s", long = "skip", default_value = "", help = "skip packages")]
    skip: String,

    #[structopt(
        short = "l",
        long = "limit",
        parse(try_from_str),
        default_value = "0",
        help = "process not more packages than limit"
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
    if !opts.tag.is_empty() {
        debug!("checking {} at {}", opts.tag, url);
        if hashfwd::is_tag_exists(ws, url, &opts.tag) {
            Some(opts.tag.to_owned())
        } else {
            None
        }
    } else if !opts.branch.is_empty() {
        debug!("search last commit on branch {}", opts.branch);
        hashfwd::get_latest_commit(ws, url, &opts.branch)
    } else {
        None
    }
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
        } else {
            warn!("can't find new version for {}", url);
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
