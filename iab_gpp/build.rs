use std::error::Error;
use std::path::{Path, PathBuf};
use std::{env, fs};
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("decode_tests.rs");

    let mut code = String::new();
    code.push_str(
        r#"use crate::common::TestCase;
"#,
    );

    for entry in list_data_files() {
        let name = entry.file_stem().unwrap().to_str().unwrap();
        let filename = &entry.file_name().unwrap().to_str().unwrap();

        code.push_str(&format!(
            "
#[test]
fn {name}() {{
    let test_case = TestCase::load_from_file(\"tests/data/{filename}\").unwrap();
    test_case.assert_json_matches();
}}
"
        ));
    }

    fs::write(dest_path, code)?;

    Ok(())
}

fn list_data_files() -> Vec<PathBuf> {
    WalkDir::new("tests/data")
        .into_iter()
        .flatten()
        .filter(|entry| entry.file_type().is_file())
        .map(|e| e.into_path())
        .collect()
}
