use std::{env, path::PathBuf};
use temple3::vectors;

fn corpus_root() -> PathBuf {
    PathBuf::from(env::var_os("TEMPLE3_CORPUS_ROOT").expect("set TEMPLE3_CORPUS_ROOT to the external corpus"))
}

#[test]
#[ignore = "requires the external conformance corpus"]
fn gate_1() {
    assert_eq!(
        vectors::kernel_vectors(&corpus_root()).unwrap(),
        117
    );
}
#[test]
#[ignore = "requires the external conformance corpus"]
fn gate_2() {
    assert_eq!(
        vectors::identity_vectors(&corpus_root()).unwrap(),
        18
    );
}
#[test]
#[ignore = "requires the external conformance corpus"]
fn gate_3() {
    assert_eq!(
        vectors::coordination_vectors(&corpus_root()).unwrap(),
        74
    );
}
#[test]
#[ignore = "requires the external conformance corpus"]
fn gate_4() {
    assert_eq!(
        vectors::federation_vectors(&corpus_root()).unwrap(),
        41
    );
}
