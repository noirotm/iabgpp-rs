use proc_macro2::TokenStream;
use quote::quote;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{env, fs};
use walkdir::WalkDir;

fn main() -> Result<(), Box<dyn Error>> {
    generate_decode_tests()
}

fn generate_decode_tests() -> Result<(), Box<dyn Error>> {
    let test_cases = find_test_cases();
    let token_stream = quote! {
        use test_case::test_case;
        #(#test_cases)*
        fn test_decode(filename: &str) {
            crate::common::TestCase::load_from_file(filename).unwrap().assert_json_matches();
        }
    };
    let syntax_tree = syn::parse2(token_stream)?;
    let pretty = prettyplease::unparse(&syntax_tree);

    let out_dir = env::var("OUT_DIR")?;
    let dest_path = Path::new(&out_dir).join("decode_tests.rs");
    fs::write(dest_path, pretty)?;

    Ok(())
}

fn find_data_files() -> impl Iterator<Item = PathBuf> {
    WalkDir::new("tests/data")
        .into_iter()
        .flatten()
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .file_name()
                    .to_str()
                    .is_some_and(|s| s.ends_with(".json"))
        })
        .map(|e| e.into_path())
}

fn find_test_cases() -> impl Iterator<Item = TokenStream> {
    find_data_files().filter_map(|entry| {
        let path = entry.to_str()?;
        let name = entry.file_stem()?.to_str()?;
        Some(quote! {
            #[test_case(#path ; #name)]
        })
    })
}
