fn main() {
    tauri_build::build();
    println!("cargo:rerun-if-changed=migrations");
}
