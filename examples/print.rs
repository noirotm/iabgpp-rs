use iab_gpp::v1::{GPPStr, SectionMapper};
use std::env::args;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let s = args().nth(1).unwrap_or("".to_string());
    let gpp_str = GPPStr::extract_from_str(&s)?;

    for &id in gpp_str.section_ids() {
        let section = gpp_str.decode_section(id)?;
        println!("{:#?}", &section);
    }

    Ok(())
}
