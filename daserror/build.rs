use rustc_version::{version, Version};

fn main() {
    let v = version().unwrap();

    println!("cargo::rustc-check-cfg=cfg(ERROR_IN_STD)");
    println!("cargo::rustc-check-cfg=cfg(ERROR_IN_CORE)");

    if v >= Version::parse("1.81.0").unwrap() {
        println!("cargo:rustc-cfg=ERROR_IN_CORE");
    } else {
        println!("cargo:rustc-cfg=ERROR_IN_STD");
    }
}
