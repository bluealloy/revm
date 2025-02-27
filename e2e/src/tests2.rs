use crate::runner::execute_test_suite;
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

fn run_e2e_test(test_path: &'static str) {
    let path = format!("./{}", test_path);
    let elapsed = Arc::new(Mutex::new(Duration::new(0, 0)));
    execute_test_suite(Path::new(path.as_str()), &elapsed, false, true).unwrap();
}

macro_rules! define_tests {
    (
        $( fn $test_name:ident($test_path:literal); )*
    ) => {
        $(
            #[test]
            fn $test_name() {
                super::run_e2e_test($test_path)
            }
        )*
    };
}

mod failing_tests {
    define_tests! {
        // fn st_e_i_p4844_blobtransactions_opcode_blobh_bounds("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/opcodeBlobhBounds.json");
        // fn st_e_i_p4844_blobtransactions_opcode_blobhash_out_of_range("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/opcodeBlobhashOutOfRange.json");
        // fn revert_in_create_in_init_paris("tests/GeneralStateTests/stRevertTest/RevertInCreateInInit_Paris.json");
        // fn st_e_i_p3860_limitmeterinitcode_create_init_code_size_limit("tests/GeneralStateTests/Shanghai/stEIP3860-limitmeterinitcode/createInitCodeSizeLimit.json");
        // fn st_e_i_p3855_push0_push0("tests/GeneralStateTests/Shanghai/stEIP3855-push0/push0.json");
        // fn st_e_i_p3855_push0_push0_gas("tests/GeneralStateTests/Shanghai/stEIP3855-push0/push0Gas.json");
        // fn cancun_st_e_i_p1153_transient_storage_11_tstore_delegate_call("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/11_tstoreDelegateCall.json");
        // fn cancun_eip4844_blobs_invalid_tx_max_fee_per_blob_gas_state("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/invalid_tx_max_fee_per_blob_gas_state.json");
        fn add_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/addNonConst.json");
    }
}
