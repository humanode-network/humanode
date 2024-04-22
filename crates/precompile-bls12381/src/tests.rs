use pallet_evm_test_vector_support::{
    test_precompile_failure_test_vectors, test_precompile_test_vectors,
};

use super::*;

#[test]
fn process_consensus_tests() -> Result<(), String> {
    test_precompile_test_vectors::<Bls12381G1Add>("testdata/bls12381G1Add.json")?;
    test_precompile_test_vectors::<Bls12381G1Mul>("testdata/bls12381G1Mul.json")?;
    test_precompile_test_vectors::<Bls12381G1MultiExp>("testdata/bls12381G1MultiExp.json")?;
    test_precompile_test_vectors::<Bls12381G2Add>("testdata/bls12381G2Add.json")?;
    test_precompile_test_vectors::<Bls12381G2Mul>("testdata/bls12381G2Mul.json")?;
    test_precompile_test_vectors::<Bls12381G2MultiExp>("testdata/bls12381G2MultiExp.json")?;
    test_precompile_test_vectors::<Bls12381Pairing>("testdata/bls12381Pairing.json")?;
    test_precompile_test_vectors::<Bls12381MapG1>("testdata/bls12381MapG1.json")?;
    test_precompile_test_vectors::<Bls12381MapG2>("testdata/bls12381MapG2.json")?;
    Ok(())
}

#[test]
fn process_consensus_failure_tests() -> Result<(), String> {
    test_precompile_failure_test_vectors::<Bls12381G1Add>("testdata/fail-bls12381G1Add.json")?;
    test_precompile_failure_test_vectors::<Bls12381G1Mul>("testdata/fail-bls12381G1Mul.json")?;
    test_precompile_failure_test_vectors::<Bls12381G1MultiExp>(
        "testdata/fail-bls12381G1MultiExp.json",
    )?;
    test_precompile_failure_test_vectors::<Bls12381G2Add>("testdata/fail-bls12381G2Add.json")?;
    test_precompile_failure_test_vectors::<Bls12381G2Mul>("testdata/fail-bls12381G2Mul.json")?;
    test_precompile_failure_test_vectors::<Bls12381G2MultiExp>(
        "testdata/fail-bls12381G2MultiExp.json",
    )?;
    test_precompile_failure_test_vectors::<Bls12381Pairing>("testdata/fail-bls12381Pairing.json")?;
    test_precompile_failure_test_vectors::<Bls12381MapG1>("testdata/fail-bls12381MapG1.json")?;
    test_precompile_failure_test_vectors::<Bls12381MapG2>("testdata/fail-bls12381MapG2.json")?;
    Ok(())
}
