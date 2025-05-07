fn main() {
    println!("cargo:rustc-link-lib=static=duckdb");
    println!("cargo:rustc-cflags=/EHsc");
}