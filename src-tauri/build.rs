fn main() {
    // The Windows resource embeds icons/icon.ico; without this an incremental build keeps the old icon.
    println!("cargo:rerun-if-changed=icons/icon.ico");
    tauri_build::build()
}
