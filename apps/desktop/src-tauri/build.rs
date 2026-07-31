use std::fs;
use std::path::PathBuf;

fn ensure_frontend_dist_placeholder() {
    // `tauri::generate_context!` requires `frontendDist` to exist at compile time.
    // Dev/build scripts produce the real UI; this keeps plain `cargo test/build` working.
    let dist = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../packages/ui/dist");
    if dist.join("index.html").exists() {
        return;
    }
    let _ = fs::create_dir_all(&dist);
    let _ = fs::write(
        dist.join("index.html"),
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Visual Library</title></head><body><p>Build UI with pnpm --filter @visual-library/ui build</p></body></html>\n",
    );
}

fn main() {
    ensure_frontend_dist_placeholder();
    tauri_build::build()
}
