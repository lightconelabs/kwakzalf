use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing manifest dir"));
    let emoji_dir = manifest_dir.join("emoji-svg");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("missing out dir"));
    let output = out_dir.join("embedded_emoji.rs");

    println!("cargo:rerun-if-changed={}", emoji_dir.display());

    let mut entries = Vec::new();
    collect_svg_entries(&emoji_dir, &mut entries);
    entries.sort();

    let mut generated = String::from(
        "pub fn bundled_svg_data(name: &str) -> Option<&'static str> {\n    match name {\n",
    );
    for entry in entries {
        generated.push_str("        \"");
        generated.push_str(&entry.0);
        generated.push_str("\" => Some(include_str!(r#\"");
        generated.push_str(&entry.1);
        generated.push_str("\"#)),\n");
    }
    generated.push_str("        _ => None,\n    }\n}\n");

    fs::write(output, generated).expect("failed to write embedded emoji map");
}

fn collect_svg_entries(dir: &Path, entries: &mut Vec<(String, String)>) {
    let read_dir = fs::read_dir(dir).expect("failed to read emoji directory");
    for entry in read_dir {
        let entry = entry.expect("failed to read dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_svg_entries(&path, entries);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("svg") {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("invalid svg filename")
            .to_owned();
        let full_path = path.to_string_lossy().into_owned();
        entries.push((stem, full_path));
    }
}
