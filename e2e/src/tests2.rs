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
        // fn st_e_i_p3860_limitmeterinitcode_create_init_code_size_limit("tests/GeneralStateTests/Shanghai/stEIP3860-limitmeterinitcode/createInitCodeSizeLimit.json");
        // fn st_e_i_p3651_warmcoinbase_coinbase_warm_account_call_gas_fail("tests/GeneralStateTests/Shanghai/stEIP3651-warmcoinbase/coinbaseWarmAccountCallGasFail.json");
        // fn st_e_i_p3651_warmcoinbase_coinbase_warm_account_call_gas("tests/GeneralStateTests/Shanghai/stEIP3651-warmcoinbase/coinbaseWarmAccountCallGas.json");
        // fn cancun_st_e_i_p1153_transient_storage_08_revert_undoes_transient_store("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/08_revertUndoesTransientStore.json");
        // fn cancun_st_e_i_p1153_transient_storage_14_revert_after_nested_staticcall("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/14_revertAfterNestedStaticcall.json");
        // fn c_r_e_a_t_e2_call_data("tests/GeneralStateTests/stCreateTest/CREATE2_CallData.json");
        // fn call_contract_to_create_contract_and_call_it_o_o_g("tests/GeneralStateTests/stInitCodeTest/CallContractToCreateContractAndCallItOOG.json");
        // fn st_e_i_p3855_push0_push0("tests/GeneralStateTests/Shanghai/stEIP3855-push0/push0.json");
        // fn transaction_create_auto_suicide_contract("tests/GeneralStateTests/stInitCodeTest/TransactionCreateAutoSuicideContract.json");
        // fn call_contract_to_create_contract_o_o_g_bonus_gas("tests/GeneralStateTests/stInitCodeTest/CallContractToCreateContractOOGBonusGas.json");
        // fn create_large_result("tests/GeneralStateTests/stCreateTest/createLargeResult.json");
        // fn ext_code_copy_tests_paris("tests/GeneralStateTests/stCodeCopyTest/ExtCodeCopyTestsParis.json");
        // fn create_address_warm_after_fail("tests/GeneralStateTests/stCreateTest/CreateAddressWarmAfterFail.json");
        // fn create_results("tests/GeneralStateTests/stCreateTest/CreateResults.json");
        fn refund_tx_to_suicide_o_o_g("tests/GeneralStateTests/stRefundTest/refund_TxToSuicideOOG.json");
    }
}
