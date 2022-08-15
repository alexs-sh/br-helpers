mod mkfile;

fn main() {
    let fl = mkfile::MkFile::new("mkfiles/boost.mk");
    let info = fl.read_info().unwrap();
    println!("{:?}", info);
}
