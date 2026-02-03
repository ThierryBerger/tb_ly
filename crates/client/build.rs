fn main() {
    dotenvy::dotenv().ok();
    if let Ok(val) = std::env::var("CERT_DIGEST") {
        println!("cargo:rustc-env=CERT_DIGEST={val}");
    }
}
