use std::io::Write;

fn write_package_ps1(version: &str) {
    let file = std::fs::File::create("../package.ps1").unwrap();
    let mut writer = std::io::BufWriter::new(file);
    write!(writer, include_str!("src/template_package.ps1"), version).unwrap();
}

fn main() {
    let version = env!("CARGO_PKG_VERSION").replace(".", "_");
    std::fs::create_dir_all(format!("../package/MMAccel_{}/MMAccel", version)).unwrap();
    write_package_ps1(&version);
}
