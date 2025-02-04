use iab_gpp::v1::GPPString;
use std::env::args;
use std::str::FromStr;

fn main() {
    let s = args()
        .nth(1)
        .unwrap_or_else(|| "DBABM~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA".to_string());

    let gpp_str = GPPString::from_str(&s).expect("a valid GPP string");

    let sections = gpp_str
        .decode_all_sections()
        .into_iter()
        .flat_map(|r| r.ok())
        .collect::<Vec<_>>();

    #[cfg(feature = "serde")]
    println!("{}", serde_json::to_string_pretty(&sections).unwrap());
}
