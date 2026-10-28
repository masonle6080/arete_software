use std::fmt::Write as _;
use std::path::Path;

fn main() {
    generate_migration_list();
    tauri_build::build();
}

/// Builds `$OUT_DIR/migrations.rs` from every `migrations/*.sql` file, in file
/// name order. A new migration is one new file; no Rust list needs editing.
fn generate_migration_list() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    println!("cargo:rerun-if-changed={}", dir.display());

    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .expect("read migrations/")
        .map(|e| e.expect("read migrations/ entry").path())
        .filter(|p| p.extension().is_some_and(|e| e == "sql"))
        .collect();
    files.sort();

    let mut out = String::from("&[\n");
    for path in &files {
        let name = path.file_stem().unwrap().to_str().expect("UTF-8 file name");
        println!("cargo:rerun-if-changed={}", path.display());
        writeln!(
            out,
            "    Migration {{ name: {name:?}, sql: include_str!({:?}) }},",
            path.display().to_string()
        )
        .unwrap();
    }
    out.push(']');

    let dest = Path::new(&std::env::var("OUT_DIR").unwrap()).join("migrations.rs");
    std::fs::write(dest, out).unwrap();
}
