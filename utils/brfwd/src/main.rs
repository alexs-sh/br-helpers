mod hashfwd;

use base::{
    gitworkspace::{GitWorkspace, Options as GitWorkspaceOptions},
    mkfile,
    package::PackageReader,
    utils,
};

use log::{error, info, warn};
use std::collections::HashSet;
use std::io::{Error, ErrorKind};
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

    #[structopt(
        short = "k",
        long = "key",
        help = "path to the SSH key. Empty by default, that means $HOME/.ssh/id_rsa will be used",
        default_value = ""
    )]
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

    #[structopt(
        short = "s",
        long = "skip",
        default_value = "",
        help = "don't process packages presented here"
    )]
    skip: String,

    #[structopt(
        short = "d",
        long = "direct",
        default_value = "",
        help = "process only packages presented here"
    )]
    direct: String,

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

fn set_from_string(input: &str) -> HashSet<String> {
    input.split(',').fold(HashSet::new(), |mut acc, value| {
        if !value.is_empty() {
            acc.insert(value.to_owned());
        }
        acc
    })
}

fn set_print(prefix: &str, set: &HashSet<String>) {
    for r in set {
        info!("add {} into {}", r, prefix);
    }
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

fn with_default_key(key: &String, default: Option<String>) -> String {
    if key.is_empty() {
        if let Some(default) = default {
            return default;
        }
    }
    key.to_owned()
}

fn run(opts: Options) -> Result<(), Error> {
    let packages = guess_reader(&opts.input)?.read()?;
    let mut wsopts = GitWorkspaceOptions::new(&opts.workdir);
    let params = hashfwd::CheckPackageParameters {
        allow: set_from_string(&opts.direct),
        deny: set_from_string(&opts.skip),
    };

    set_print("denylist", &params.deny);
    set_print("allowlist", &params.allow);

    wsopts.key = with_default_key(&opts.key, utils::get_default_ssh_key());
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
        if !hashfwd::check_package(package, &params) {
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

fn check_opts(opts: Options) -> Result<Options, Error> {
    if !(opts.direct.is_empty() || opts.skip.is_empty()) {
        println!("'skip' and 'direct' parameters are both set at the same time. Please choose only one of them.");
        return Err(Error::new(ErrorKind::InvalidInput, "invalid parameters"));
    }
    Ok(opts)
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let opts = check_opts(Options::from_args())?;
    run(opts)
        .map(|_| {
            println!("Done");
        })
        .map_err(|err| {
            error!("brfwd failed:{:?}", err);
            err
        })
}
