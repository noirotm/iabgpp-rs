use iab_gpp::v1::GPPString;
use std::env::args;
use std::error::Error;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let s = args().nth(1).unwrap_or("".to_string());
    let gpp_str = GPPString::from_str(&s)?;

    for &id in gpp_str.section_ids() {
        let section = gpp_str.decode_section(id)?;
        println!("{:#?}", &section);
    }

    Ok(())
}
