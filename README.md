# iabgpp-rs

[![Build](https://github.com/noirotm/iabgpp-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/noirotm/iabgpp-rs/actions/workflows/rust.yml)
[![docs.rs](https://img.shields.io/docsrs/iab_gpp)](https://docs.rs/iab_gpp/latest/iab_gpp/)
[![Latest Version](https://img.shields.io/crates/v/iab_gpp.svg)](https://crates.io/crates/iab_gpp)

Rust implementation of the IAB Global Privacy Platform (GPP)
[consent string specification](https://github.com/InteractiveAdvertisingBureau/Global-Privacy-Platform/blob/main/Core/Consent%20String%20Specification.md).

⚠️ This is work in progress.

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

## GPP Standard support

This crate intends to be in sync with the GPP specification, meaning that it should
be able to read payloads with the versions specified here.

If the standard gets updated with new versions, this page should keep track of
incompatibilities.

Legend:

- ✅ complete support
- ➖ partial support
- 🧪 experimental support
- ❌ no support

| Section                      | Reading | Writing |
|------------------------------|:-------:|:-------:|
| GPP string v1                |    ✅    |    ❌    |
| US Privacy v1 (deprecated)   |    ✅    |    ❌    |
| EU TCF v2.2                  |    ✅    |    ❌    |
| EU TCF v1 (deprecated)       |    ✅    |    ❌    |
| Canadian TCF v1 (deprecated) |    ✅    |    ❌    |
| Canadian TCF v1.1            |    ✅    |    ❌    |
| US - National v1             |    ✅    |    ❌    |
| US - National v2             |   🧪    |    ❌    |
| US - California              |    ✅    |    ❌    |
| US - Virginia                |    ✅    |    ❌    |
| US - Colorado                |    ✅    |    ❌    |
| US - Utah                    |    ✅    |    ❌    |
| US - Connecticut             |    ✅    |    ❌    |
| US - Florida                 |   🧪    |    ❌    |
| US - Montana                 |   🧪    |    ❌    |
| US - Oregon                  |   🧪    |    ❌    |
| US - Texas                   |   🧪    |    ❌    |
| US - Delaware                |   🧪    |    ❌    |
| US - Iowa                    |   🧪    |    ❌    |
| US - Nebraska                |   🧪    |    ❌    |
| US - New Hampshire           |   🧪    |    ❌    |
| US - New Jersey              |   🧪    |    ❌    |
| US - Tennessee               |   🧪    |    ❌    |
| US - Minnesota               |   🧪    |    ❌    |
| US - Maryland                |   🧪    |    ❌    |
| US - Indiana                 |   🧪    |    ❌    |
| US - Kentucky                |   🧪    |    ❌    |

## Development status

The current plan:

- complete reader implementation for version 0.1
- read + write support in version 0.2
