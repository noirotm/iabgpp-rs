use iab_gpp::v1::{GPPStr, SectionMapper};

fn main() {
    let s = "DBABM~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA";
    let gpp_str = GPPStr::extract_from_str(s).expect("a valid GPP string");

    for &id in gpp_str.section_ids() {
        println!("Section id: {:?}", id);

        let section = gpp_str.decode_section(id).expect("a valid section");
        println!("Section: {:?}", &section);
    }
}
