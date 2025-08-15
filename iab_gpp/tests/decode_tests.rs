mod common;

// Generated tests: one per file present in the data folder
include!(concat!(env!("OUT_DIR"), "/decode_tests.rs"));
