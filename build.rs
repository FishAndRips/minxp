use std::env;

fn main() {
    // same thing as what rust's standard library does
    println!("cargo:rustc-env=MINXP_ENV_ARCH={}", env::var("CARGO_CFG_TARGET_ARCH").unwrap());
}
