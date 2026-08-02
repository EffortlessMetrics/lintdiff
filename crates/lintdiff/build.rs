fn main() {
    if let Ok(sha) = std::env::var("GIT_SHA") {
        // Validate that GIT_SHA contains only valid hex characters to prevent
        // cargo instruction injection. A valid git SHA is 40 hex characters.
        if sha.chars().all(|c| c.is_ascii_hexdigit()) && !sha.is_empty() {
            println!("cargo:rustc-env=GIT_SHA={sha}");
        }
    }
    println!("cargo:rerun-if-env-changed=GIT_SHA");
}
