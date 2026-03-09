fn main() {
    if let Ok(sha) = std::env::var("GIT_SHA") {
        println!("cargo:rustc-env=GIT_SHA={sha}");
    }
    println!("cargo:rerun-if-env-changed=GIT_SHA");
}
