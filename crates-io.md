# iab_gpp

Rust implementation of the IAB Global Privacy Platform (GPP)
[consent string specification](https://github.com/InteractiveAdvertisingBureau/Global-Privacy-Platform/blob/main/Core/Consent%20String%20Specification.md).

## Features

- Eager or lazy decoding of GPP sections
- Owning type (GPPString)
- Read support for all current GPP sections

## Usage example

Cargo.toml:

```toml
[dependencies]
iab-gpp = "0.1"
```

main.rs:

```rust
use std::error::Error;
use std::str::FromStr;
use iab_gpp::v1::GPPString;

fn main() {
    let s = "DBABM~CPXxRfAPXxRfAAfKABENB-CgAAAAAAAAAAYgAAAAAAAA";
    let gpp_str = GPPString::from_str(s).expect("a valid GPP string");

    for &id in gpp_str.section_ids() {
        println!("Section id: {:?}", id);

        let section = gpp_str.decode_section(id).expect("a valid section");
        println!("Section: {:?}", &section);
    }
}
```
