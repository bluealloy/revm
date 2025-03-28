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
        // fn refund_tx_to_suicide_o_o_g("tests/GeneralStateTests/stRefundTest/refund_TxToSuicideOOG.json");
        // fn call_to_suicide_then_extcodehash("tests/GeneralStateTests/stExtCodeHash/callToSuicideThenExtcodehash.json");
        // fn code_copy_zero_paris("tests/GeneralStateTests/stExtCodeHash/codeCopyZero_Paris.json");
        // fn ext_code_copy_bounds("tests/GeneralStateTests/stExtCodeHash/extCodeCopyBounds.json");
        // fn create_fail_result("tests/GeneralStateTests/stCreateTest/createFailResult.json");
        // fn call_outsize_then_create_successful_then_returndatasize("tests/GeneralStateTests/stReturnDataTest/call_outsize_then_create_successful_then_returndatasize.json");
        // fn call_then_create_successful_then_returndatasize("tests/GeneralStateTests/stReturnDataTest/call_then_create_successful_then_returndatasize.json");
        // fn revert_ret_data_size("tests/GeneralStateTests/stReturnDataTest/revertRetDataSize.json");
        // fn sstore_call_to_self_sub_refund_below_zero("tests/GeneralStateTests/stSStoreTest/SstoreCallToSelfSubRefundBelowZero.json");
        // fn sstore_change_from_external_call_in_init_code("tests/GeneralStateTests/stSStoreTest/sstore_changeFromExternalCallInInitCode.json");
        // fn sstore_gas("tests/GeneralStateTests/stSStoreTest/sstoreGas.json");
        // fn sstore_xto0to_x("tests/GeneralStateTests/stSStoreTest/sstore_Xto0toX.json");
        // fn sstore_xto0to_xto0("tests/GeneralStateTests/stSStoreTest/sstore_Xto0toXto0.json");
        // fn sstore_xto0to_y("tests/GeneralStateTests/stSStoreTest/sstore_Xto0toY.json");
        // fn sstore_gas_left("tests/GeneralStateTests/stSStoreTest/sstore_gasLeft.json");


        // fn st_static_call_static_loop_calls_depth_then_revert2("tests/GeneralStateTests/stStaticCall/static_LoopCallsDepthThenRevert2.json");
        // fn st_revert_test_loop_calls_depth_then_revert3("tests/GeneralStateTests/stRevertTest/LoopCallsDepthThenRevert3.json");

        // fn cancun_st_e_i_p1153_transient_storage_15_tstore_cannot_be_dosd("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/15_tstoreCannotBeDosd.json");
        // fn cancun_st_e_i_p1153_transient_storage_21_tstore_cannot_be_dosd_o_o_o("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/21_tstoreCannotBeDosdOOO.json");

        // fn st_e_i_p4844_blobtransactions_blobhash_list_bounds3("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds3.json");
        // fn st_e_i_p4844_blobtransactions_blobhash_list_bounds4("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds4.json");
        // fn st_e_i_p4844_blobtransactions_blobhash_list_bounds5("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds5.json");
        // fn st_e_i_p4844_blobtransactions_blobhash_list_bounds6("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds6.json");
        // fn cancun_eip4844_blobs_blob_tx_attribute_gasprice_opcode("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blob_tx_attribute_gasprice_opcode.json");
        // fn cancun_eip4844_blobs_invalid_tx_max_fee_per_blob_gas_state("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/invalid_tx_max_fee_per_blob_gas_state.json");
        // fn st_attack_test_contract_creation_spam("tests/GeneralStateTests/stAttackTest/ContractCreationSpam.json");
        // fn bad_opcodes("tests/GeneralStateTests/stBadOpcode/badOpcodes.json");
        // fn opc0_c_diff_places("tests/GeneralStateTests/stBadOpcode/opc0CDiffPlaces.json");
        // fn undefined_opcode_first_byte("tests/GeneralStateTests/stBadOpcode/undefinedOpcodeFirstByte.json");
        // fn call1024_pre_calls("tests/GeneralStateTests/stCallCreateCallCodeTest/Call1024PreCalls.json");
        // fn call_lose_gas_o_o_g("tests/GeneralStateTests/stCallCreateCallCodeTest/CallLoseGasOOG.json");
        // fn call_recursive_bomb_pre_call("tests/GeneralStateTests/stCallCreateCallCodeTest/CallRecursiveBombPreCall.json");
        // fn callcode1024_balance_too_low("tests/GeneralStateTests/stCallCreateCallCodeTest/Callcode1024BalanceTooLow.json");
        // fn call_outsize_then_create2_successful_then_returndatasize("tests/GeneralStateTests/stCreate2/call_outsize_then_create2_successful_then_returndatasize.json");
        // fn call_then_create2_successful_then_returndatasize("tests/GeneralStateTests/stCreate2/call_then_create2_successful_then_returndatasize.json");
        // fn create2_on_depth1023("tests/GeneralStateTests/stCreate2/Create2OnDepth1023.json");
        // fn create2_on_depth1024("tests/GeneralStateTests/stCreate2/Create2OnDepth1024.json");
        // fn create2_recursive("tests/GeneralStateTests/stCreate2/Create2Recursive.json");

        // fn st_delegatecall_test_homestead_call1024_pre_calls("tests/GeneralStateTests/stDelegatecallTestHomestead/Call1024PreCalls.json");
        // fn st_delegatecall_test_homestead_call_lose_gas_o_o_g("tests/GeneralStateTests/stDelegatecallTestHomestead/CallLoseGasOOG.json");
        // fn st_delegatecall_test_homestead_call_recursive_bomb_pre_call("tests/GeneralStateTests/stDelegatecallTestHomestead/CallRecursiveBombPreCall.json");
        // fn st_delegatecall_test_homestead_delegatecall1024("tests/GeneralStateTests/stDelegatecallTestHomestead/Delegatecall1024.json");

        // fn call50000("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000.json");
        // fn call50000_ecrec("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000_ecrec.json");
        // fn call50000_identity("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000_identity.json");
        // fn call50000_identity2("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000_identity2.json");
        // fn call50000_rip160("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000_rip160.json");
        // fn callcode50000("tests/GeneralStateTests/stQuadraticComplexityTest/Callcode50000.json");
        // fn return50000("tests/GeneralStateTests/stQuadraticComplexityTest/Return50000.json");
        // fn return50000_2("tests/GeneralStateTests/stQuadraticComplexityTest/Return50000_2.json");

        // fn loop_calls_depth_then_revert2("tests/GeneralStateTests/stRevertTest/LoopCallsDepthThenRevert2.json");
        // fn loop_calls_depth_then_revert3("tests/GeneralStateTests/stRevertTest/LoopCallsDepthThenRevert3.json");

        // fn underflow_test("tests/GeneralStateTests/stStackTests/underflowTest.json");

        // fn static_call10("tests/GeneralStateTests/stStaticCall/static_Call10.json");
        // fn static_call1024_pre_calls("tests/GeneralStateTests/stStaticCall/static_Call1024PreCalls.json");
        // fn static_call1024_pre_calls2("tests/GeneralStateTests/stStaticCall/static_Call1024PreCalls2.json");
        // fn static_call1024_pre_calls3("tests/GeneralStateTests/stStaticCall/static_Call1024PreCalls3.json");
        // fn static_call50000("tests/GeneralStateTests/stStaticCall/static_Call50000.json");
        // fn static_call50000_ecrec("tests/GeneralStateTests/stStaticCall/static_Call50000_ecrec.json");
        // fn static_call50000_identity("tests/GeneralStateTests/stStaticCall/static_Call50000_identity.json");
        // fn static_call50000_identity2("tests/GeneralStateTests/stStaticCall/static_Call50000_identity2.json");
        // fn static_call50000_rip160("tests/GeneralStateTests/stStaticCall/static_Call50000_rip160.json");
        // fn static_call_lose_gas_o_o_g("tests/GeneralStateTests/stStaticCall/static_CallLoseGasOOG.json");
        // fn static_call_recursive_bomb_pre_call("tests/GeneralStateTests/stStaticCall/static_CallRecursiveBombPreCall.json");
        // fn static_call_recursive_bomb_pre_call2("tests/GeneralStateTests/stStaticCall/static_CallRecursiveBombPreCall2.json");
        // fn static_loop_calls_depth_then_revert2("tests/GeneralStateTests/stStaticCall/static_LoopCallsDepthThenRevert2.json");
        // fn static_loop_calls_depth_then_revert3("tests/GeneralStateTests/stStaticCall/static_LoopCallsDepthThenRevert3.json");
        // fn static_loop_calls_then_revert("tests/GeneralStateTests/stStaticCall/static_LoopCallsThenRevert.json");
        // fn static_return50000_2("tests/GeneralStateTests/stStaticCall/static_Return50000_2.json");

        // fn call10("tests/GeneralStateTests/stSystemOperationsTest/Call10.json");
        // fn call_to_name_registrator_not_much_memory1("tests/GeneralStateTests/stSystemOperationsTest/CallToNameRegistratorNotMuchMemory1.json");
    }
}
