//! With the `duckdb` feature, the linker looks for libduckdb in DUCKDB_LIB_DIR.

fn main() {
    println!("cargo:rerun-if-env-changed=DUCKDB_LIB_DIR");
    if std::env::var_os("CARGO_FEATURE_DUCKDB").is_some()
        && let Some(dir) = std::env::var_os("DUCKDB_LIB_DIR")
    {
        println!("cargo:rustc-link-search=native={}", dir.to_string_lossy());
    }
}
