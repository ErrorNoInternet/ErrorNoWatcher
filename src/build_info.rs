pub mod built {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub fn version_formatted() -> String {
    format!(
        "v{} ({})",
        env!("CARGO_PKG_VERSION"),
        built::GIT_COMMIT_HASH_SHORT.unwrap_or("unknown commit")
    )
}
