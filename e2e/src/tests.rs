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

mod good_coverage_tests {
    define_tests! {
        fn st_e_i_p3860_limitmeterinitcode_create_init_code_size_limit("tests/GeneralStateTests/Shanghai/stEIP3860-limitmeterinitcode/createInitCodeSizeLimit.json");
        fn st_e_i_p3651_warmcoinbase_coinbase_warm_account_call_gas_fail("tests/GeneralStateTests/Shanghai/stEIP3651-warmcoinbase/coinbaseWarmAccountCallGasFail.json");
        fn st_e_i_p3651_warmcoinbase_coinbase_warm_account_call_gas("tests/GeneralStateTests/Shanghai/stEIP3651-warmcoinbase/coinbaseWarmAccountCallGas.json");
        fn cancun_st_e_i_p1153_transient_storage_08_revert_undoes_transient_store("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/08_revertUndoesTransientStore.json");
        fn cancun_st_e_i_p1153_transient_storage_14_revert_after_nested_staticcall("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/14_revertAfterNestedStaticcall.json");
        fn c_r_e_a_t_e2_call_data("tests/GeneralStateTests/stCreateTest/CREATE2_CallData.json");
        fn call_contract_to_create_contract_and_call_it_o_o_g("tests/GeneralStateTests/stInitCodeTest/CallContractToCreateContractAndCallItOOG.json");
        fn st_e_i_p3855_push0_push0("tests/GeneralStateTests/Shanghai/stEIP3855-push0/push0.json");
        fn transaction_create_auto_suicide_contract("tests/GeneralStateTests/stInitCodeTest/TransactionCreateAutoSuicideContract.json");
        fn call_contract_to_create_contract_o_o_g_bonus_gas("tests/GeneralStateTests/stInitCodeTest/CallContractToCreateContractOOGBonusGas.json");
        fn create_large_result("tests/GeneralStateTests/stCreateTest/createLargeResult.json");
        fn ext_code_copy_tests_paris("tests/GeneralStateTests/stCodeCopyTest/ExtCodeCopyTestsParis.json");
        fn create_address_warm_after_fail("tests/GeneralStateTests/stCreateTest/CreateAddressWarmAfterFail.json");
        fn create_results("tests/GeneralStateTests/stCreateTest/CreateResults.json");
        fn refund_tx_to_suicide_o_o_g("tests/GeneralStateTests/stRefundTest/refund_TxToSuicideOOG.json");
        fn call_to_suicide_then_extcodehash("tests/GeneralStateTests/stExtCodeHash/callToSuicideThenExtcodehash.json");
        fn code_copy_zero_paris("tests/GeneralStateTests/stExtCodeHash/codeCopyZero_Paris.json");
        fn ext_code_copy_bounds("tests/GeneralStateTests/stExtCodeHash/extCodeCopyBounds.json");
        fn create_fail_result("tests/GeneralStateTests/stCreateTest/createFailResult.json");
        fn call_outsize_then_create_successful_then_returndatasize("tests/GeneralStateTests/stReturnDataTest/call_outsize_then_create_successful_then_returndatasize.json");
        fn call_then_create_successful_then_returndatasize("tests/GeneralStateTests/stReturnDataTest/call_then_create_successful_then_returndatasize.json");
        fn revert_ret_data_size("tests/GeneralStateTests/stReturnDataTest/revertRetDataSize.json");
        fn sstore_call_to_self_sub_refund_below_zero("tests/GeneralStateTests/stSStoreTest/SstoreCallToSelfSubRefundBelowZero.json");
        fn sstore_change_from_external_call_in_init_code("tests/GeneralStateTests/stSStoreTest/sstore_changeFromExternalCallInInitCode.json");
        fn sstore_gas("tests/GeneralStateTests/stSStoreTest/sstoreGas.json");
        fn suicides_and_internl_call_suicides_o_o_g("tests/GeneralStateTests/stTransactionTest/SuicidesAndInternlCallSuicidesOOG.json");
        fn call_ecrecover0_no_gas("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecover0_NoGas.json");
        fn create2collision_selfdestructed("tests/GeneralStateTests/stCreate2/create2collisionSelfdestructed.json");
        fn call_ecrecover1("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecover1.json");
        fn c_a_l_l_blake2f("tests/GeneralStateTests/stPreCompiledContracts2/CALLBlake2f.json");
        fn c_a_l_l_c_o_d_e_blake2f("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEBlake2f.json");
        fn c_a_l_l_c_o_d_e_ecrecover0("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecover0.json");
        fn modexp_0_0_0_20500("tests/GeneralStateTests/stPreCompiledContracts2/modexp_0_0_0_20500.json");
        fn ecadd_00_0_0_21000_0("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_0-0_21000_0.json");
        fn frontier_opcodes_value_transfer_gas_calculation("tests/GeneralStateTests/Pyspecs/frontier/opcodes/value_transfer_gas_calculation.json");
        fn c_r_e_a_t_e2_suicide("tests/GeneralStateTests/stCreate2/CREATE2_Suicide.json");
        fn selfdestruct_e_i_p2929("tests/GeneralStateTests/stSpecialTest/selfdestructEIP2929.json");
        fn id_precomps("tests/GeneralStateTests/stPreCompiledContracts/idPrecomps.json");
    }
}

mod failing_tests {
    define_tests! {
        // fn cancun_eip4844_blobs_blob_tx_attribute_gasprice_opcode("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blob_tx_attribute_gasprice_opcode.json");
        // fn cancun_eip4844_blobs_invalid_tx_max_fee_per_blob_gas_state("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/invalid_tx_max_fee_per_blob_gas_state.json");

        // this test can't pass,
        // because our precompiled contract (0x01 - ecrecover) is preloaded
        // and this test expects them to be preloaded as-well,
        // this one we can't solve ;(
        // fn ext_code_hash_dynamic_argument("tests/GeneralStateTests/stExtCodeHash/extCodeHashDynamicArgument.json");

        // you can CALL the precompiled contract,
        // it means that it can be created in the state,
        // and it affects next call gas price that is impossible to track in our model
        // this one we can't solve ;(
        // fn precomps_e_i_p2929_cancun("tests/GeneralStateTests/stPreCompiledContracts/precompsEIP2929Cancun.json");

        // this test can't pass because it modifies our precompiled contracts (0x01, 0x02, 0x03),
        // the same problem as above
        // fn self_destruct("tests/GeneralStateTests/stSolidityTest/SelfDestruct.json");

        // this test can't pass because it relays on modified EVM precompiled contract,
        // and it causes gas miscalculation
        // fn random_statetest650("tests/GeneralStateTests/stRandom2/randomStatetest650.json");

        // quadratic complexity tests, fails because of OOM, need to investigate
        // fn call50000("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000.json");
        // fn call50000_ecrec("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000_ecrec.json");
        // fn call50000_identity("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000_identity.json");
        // fn call50000_identity2("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000_identity2.json");
        // fn call50000_rip160("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000_rip160.json");
        // fn callcode50000("tests/GeneralStateTests/stQuadraticComplexityTest/Callcode50000.json");
        // fn return50000("tests/GeneralStateTests/stQuadraticComplexityTest/Return50000.json");
        // fn return50000_2("tests/GeneralStateTests/stQuadraticComplexityTest/Return50000_2.json");
        // fn static_call50000("tests/GeneralStateTests/stStaticCall/static_Call50000.json");
        // fn static_call50000_ecrec("tests/GeneralStateTests/stStaticCall/static_Call50000_ecrec.json");
        // fn static_call50000_identity("tests/GeneralStateTests/stStaticCall/static_Call50000_identity.json");
        // fn static_call50000_identity2("tests/GeneralStateTests/stStaticCall/static_Call50000_identity2.json");
        // fn static_call50000_rip160("tests/GeneralStateTests/stStaticCall/static_Call50000_rip160.json");
        // fn static_loop_calls_depth_then_revert2("tests/GeneralStateTests/stStaticCall/static_LoopCallsDepthThenRevert2.json");
        // fn static_loop_calls_depth_then_revert3("tests/GeneralStateTests/stStaticCall/static_LoopCallsDepthThenRevert3.json");
        // fn static_loop_calls_then_revert("tests/GeneralStateTests/stStaticCall/static_LoopCallsThenRevert.json");
        // fn static_return50000_2("tests/GeneralStateTests/stStaticCall/static_Return50000_2.json");
    }
}

mod st_time_consuming {
    define_tests! {

        // --- ALL PASS (run with commented debug log) ---
        fn sstore_combinations_initial01_2_paris("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial01_2_Paris.json");
        fn sstore_combinations_initial00_paris("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial00_Paris.json");
        fn sstore_combinations_initial11_paris("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial11_Paris.json");
        fn sstore_combinations_initial00_2_paris("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial00_2_Paris.json");
        fn static_call50000_sha256("tests/GeneralStateTests/stTimeConsuming/static_Call50000_sha256.json");
        fn sstore_combinations_initial20_2("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial20_2.json");
        fn sstore_combinations_initial00_2("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial00_2.json");
        fn sstore_combinations_initial10_2("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial10_2.json");
        fn sstore_combinations_initial21_paris("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial21_Paris.json");
        fn c_a_l_l_blake2f_max_rounds("tests/GeneralStateTests/stTimeConsuming/CALLBlake2f_MaxRounds.json");
        fn sstore_combinations_initial21("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial21.json");
        fn sstore_combinations_initial01("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial01.json");
        fn sstore_combinations_initial21_2("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial21_2.json");
        fn sstore_combinations_initial01_paris("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial01_Paris.json");
        fn sstore_combinations_initial00("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial00.json");
        fn sstore_combinations_initial10_paris("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial10_Paris.json");
        fn sstore_combinations_initial20("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial20.json");
        fn sstore_combinations_initial01_2("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial01_2.json");
        fn sstore_combinations_initial11("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial11.json");
        fn sstore_combinations_initial20_paris("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial20_Paris.json");
        fn sstore_combinations_initial10_2_paris("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial10_2_Paris.json");
        fn sstore_combinations_initial21_2_paris("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial21_2_Paris.json");
        fn sstore_combinations_initial11_2_paris("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial11_2_Paris.json");
        fn sstore_combinations_initial11_2("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial11_2.json");
        fn sstore_combinations_initial20_2_paris("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial20_2_Paris.json");
        fn sstore_combinations_initial10("tests/GeneralStateTests/stTimeConsuming/sstore_combinations_initial10.json");
    }
}

mod st_random {
    define_tests! {

        // -- ALL PASS ---
        fn random_statetest248("tests/GeneralStateTests/stRandom/randomStatetest248.json");
        fn random_statetest53("tests/GeneralStateTests/stRandom/randomStatetest53.json");
        fn random_statetest341("tests/GeneralStateTests/stRandom/randomStatetest341.json");
        fn random_statetest307("tests/GeneralStateTests/stRandom/randomStatetest307.json");
        fn random_statetest154("tests/GeneralStateTests/stRandom/randomStatetest154.json");
        fn random_statetest178("tests/GeneralStateTests/stRandom/randomStatetest178.json");
        fn random_statetest159("tests/GeneralStateTests/stRandom/randomStatetest159.json");
        fn random_statetest85("tests/GeneralStateTests/stRandom/randomStatetest85.json");
        fn random_statetest306("tests/GeneralStateTests/stRandom/randomStatetest306.json");
        fn random_statetest146("tests/GeneralStateTests/stRandom/randomStatetest146.json");
        fn random_statetest150("tests/GeneralStateTests/stRandom/randomStatetest150.json");
        fn random_statetest48("tests/GeneralStateTests/stRandom/randomStatetest48.json");
        fn random_statetest100("tests/GeneralStateTests/stRandom/randomStatetest100.json");
        fn random_statetest205("tests/GeneralStateTests/stRandom/randomStatetest205.json");
        fn random_statetest16("tests/GeneralStateTests/stRandom/randomStatetest16.json");
        fn random_statetest380("tests/GeneralStateTests/stRandom/randomStatetest380.json");
        fn random_statetest103("tests/GeneralStateTests/stRandom/randomStatetest103.json");
        fn random_statetest338("tests/GeneralStateTests/stRandom/randomStatetest338.json");
        fn random_statetest41("tests/GeneralStateTests/stRandom/randomStatetest41.json");
        fn random_statetest292("tests/GeneralStateTests/stRandom/randomStatetest292.json");
        fn random_statetest57("tests/GeneralStateTests/stRandom/randomStatetest57.json");
        fn random_statetest142("tests/GeneralStateTests/stRandom/randomStatetest142.json");
        fn random_statetest379("tests/GeneralStateTests/stRandom/randomStatetest379.json");
        fn random_statetest115("tests/GeneralStateTests/stRandom/randomStatetest115.json");
        fn random_statetest247("tests/GeneralStateTests/stRandom/randomStatetest247.json");
        fn random_statetest302("tests/GeneralStateTests/stRandom/randomStatetest302.json");
        fn random_statetest210("tests/GeneralStateTests/stRandom/randomStatetest210.json");
        fn random_statetest355("tests/GeneralStateTests/stRandom/randomStatetest355.json");
        fn random_statetest139("tests/GeneralStateTests/stRandom/randomStatetest139.json");
        fn random_statetest206("tests/GeneralStateTests/stRandom/randomStatetest206.json");
        fn random_statetest343("tests/GeneralStateTests/stRandom/randomStatetest343.json");
        fn random_statetest82("tests/GeneralStateTests/stRandom/randomStatetest82.json");
        fn random_statetest251("tests/GeneralStateTests/stRandom/randomStatetest251.json");
        fn random_statetest197("tests/GeneralStateTests/stRandom/randomStatetest197.json");
        fn random_statetest363("tests/GeneralStateTests/stRandom/randomStatetest363.json");
        fn random_statetest0("tests/GeneralStateTests/stRandom/randomStatetest0.json");
        fn random_statetest226("tests/GeneralStateTests/stRandom/randomStatetest226.json");
        fn random_statetest158("tests/GeneralStateTests/stRandom/randomStatetest158.json");
        fn random_statetest334("tests/GeneralStateTests/stRandom/randomStatetest334.json");
        fn random_statetest271("tests/GeneralStateTests/stRandom/randomStatetest271.json");
        fn random_statetest322("tests/GeneralStateTests/stRandom/randomStatetest322.json");
        fn random_statetest288("tests/GeneralStateTests/stRandom/randomStatetest288.json");
        fn random_statetest267("tests/GeneralStateTests/stRandom/randomStatetest267.json");
        fn random_statetest119("tests/GeneralStateTests/stRandom/randomStatetest119.json");
        fn random_statetest230("tests/GeneralStateTests/stRandom/randomStatetest230.json");
        fn random_statetest162("tests/GeneralStateTests/stRandom/randomStatetest162.json");
        fn random_statetest98("tests/GeneralStateTests/stRandom/randomStatetest98.json");
        fn random_statetest77("tests/GeneralStateTests/stRandom/randomStatetest77.json");
        fn random_statetest135("tests/GeneralStateTests/stRandom/randomStatetest135.json");
        fn random_statetest359("tests/GeneralStateTests/stRandom/randomStatetest359.json");
        fn random_statetest20("tests/GeneralStateTests/stRandom/randomStatetest20.json");
        fn random_statetest36("tests/GeneralStateTests/stRandom/randomStatetest36.json");
        fn random_statetest174("tests/GeneralStateTests/stRandom/randomStatetest174.json");
        fn random_statetest318("tests/GeneralStateTests/stRandom/randomStatetest318.json");
        fn random_statetest60("tests/GeneralStateTests/stRandom/randomStatetest60.json");
        fn random_statetest175("tests/GeneralStateTests/stRandom/randomStatetest175.json");
        fn random_statetest37("tests/GeneralStateTests/stRandom/randomStatetest37.json");
        fn random_statetest122("tests/GeneralStateTests/stRandom/randomStatetest122.json");
        fn random_statetest358("tests/GeneralStateTests/stRandom/randomStatetest358.json");
        fn random_statetest134("tests/GeneralStateTests/stRandom/randomStatetest134.json");
        fn random_statetest163("tests/GeneralStateTests/stRandom/randomStatetest163.json");
        fn random_statetest231("tests/GeneralStateTests/stRandom/randomStatetest231.json");
        fn random_statetest118("tests/GeneralStateTests/stRandom/randomStatetest118.json");
        fn random_statetest323("tests/GeneralStateTests/stRandom/randomStatetest323.json");
        fn random_statetest266("tests/GeneralStateTests/stRandom/randomStatetest266.json");
        fn random_statetest335("tests/GeneralStateTests/stRandom/randomStatetest335.json");
        fn random_statetest270("tests/GeneralStateTests/stRandom/randomStatetest270.json");
        fn random_statetest362("tests/GeneralStateTests/stRandom/randomStatetest362.json");
        fn random_statetest1("tests/GeneralStateTests/stRandom/randomStatetest1.json");
        fn random_statetest227("tests/GeneralStateTests/stRandom/randomStatetest227.json");
        fn random_statetest196("tests/GeneralStateTests/stRandom/randomStatetest196.json");
        fn random_statetest179("tests/GeneralStateTests/stRandom/randomStatetest179.json");
        fn random_statetest250("tests/GeneralStateTests/stRandom/randomStatetest250.json");
        fn random_statetest83("tests/GeneralStateTests/stRandom/randomStatetest83.json");
        fn random_statetest315("tests/GeneralStateTests/stRandom/randomStatetest315.json");
        fn random_statetest207("tests/GeneralStateTests/stRandom/randomStatetest207.json");
        fn random_statetest342("tests/GeneralStateTests/stRandom/randomStatetest342.json");
        fn random_statetest138("tests/GeneralStateTests/stRandom/randomStatetest138.json");
        fn random_statetest211("tests/GeneralStateTests/stRandom/randomStatetest211.json");
        fn random_statetest354("tests/GeneralStateTests/stRandom/randomStatetest354.json");
        fn random_statetest180("tests/GeneralStateTests/stRandom/randomStatetest180.json");
        fn random_statetest95("tests/GeneralStateTests/stRandom/randomStatetest95.json");
        fn random_statetest246("tests/GeneralStateTests/stRandom/randomStatetest246.json");
        fn random_statetest303("tests/GeneralStateTests/stRandom/randomStatetest303.json");
        fn random_statetest114("tests/GeneralStateTests/stRandom/randomStatetest114.json");
        fn random_statetest378("tests/GeneralStateTests/stRandom/randomStatetest378.json");
        fn random_statetest143("tests/GeneralStateTests/stRandom/randomStatetest143.json");
        fn random_statetest285("tests/GeneralStateTests/stRandom/randomStatetest285.json");
        fn random_statetest155("tests/GeneralStateTests/stRandom/randomStatetest155.json");
        fn random_statetest339("tests/GeneralStateTests/stRandom/randomStatetest339.json");
        fn random_statetest293("tests/GeneralStateTests/stRandom/randomStatetest293.json");
        fn random_statetest102("tests/GeneralStateTests/stRandom/randomStatetest102.json");
        fn random_statetest17("tests/GeneralStateTests/stRandom/randomStatetest17.json");
        fn random_statetest381("tests/GeneralStateTests/stRandom/randomStatetest381.json");
        fn random_statetest6("tests/GeneralStateTests/stRandom/randomStatetest6.json");
        fn random_statetest365("tests/GeneralStateTests/stRandom/randomStatetest365.json");
        fn random_statetest220("tests/GeneralStateTests/stRandom/randomStatetest220.json");
        fn random_statetest298("tests/GeneralStateTests/stRandom/randomStatetest298.json");
        fn random_statetest332("tests/GeneralStateTests/stRandom/randomStatetest332.json");
        fn random_statetest148("tests/GeneralStateTests/stRandom/randomStatetest148.json");
        fn random_statetest261("tests/GeneralStateTests/stRandom/randomStatetest261.json");
        fn random_statetest236("tests/GeneralStateTests/stRandom/randomStatetest236.json");
        fn random_statetest164("tests/GeneralStateTests/stRandom/randomStatetest164.json");
        fn random_statetest308("tests/GeneralStateTests/stRandom/randomStatetest308.json");
        fn random_statetest133("tests/GeneralStateTests/stRandom/randomStatetest133.json");
        fn random_statetest26("tests/GeneralStateTests/stRandom/randomStatetest26.json");
        fn random_statetest125("tests/GeneralStateTests/stRandom/randomStatetest125.json");
        fn random_statetest30("tests/GeneralStateTests/stRandom/randomStatetest30.json");
        fn random_statetest349("tests/GeneralStateTests/stRandom/randomStatetest349.json");
        fn random_statetest172("tests/GeneralStateTests/stRandom/randomStatetest172.json");
        fn random_statetest88("tests/GeneralStateTests/stRandom/randomStatetest88.json");
        fn random_statetest67("tests/GeneralStateTests/stRandom/randomStatetest67.json");
        fn random_statetest10("tests/GeneralStateTests/stRandom/randomStatetest10.json");
        fn random_statetest369("tests/GeneralStateTests/stRandom/randomStatetest369.json");
        fn random_statetest105("tests/GeneralStateTests/stRandom/randomStatetest105.json");
        fn random_statetest47("tests/GeneralStateTests/stRandom/randomStatetest47.json");
        fn random_statetest294("tests/GeneralStateTests/stRandom/randomStatetest294.json");
        fn random_statetest282("tests/GeneralStateTests/stRandom/randomStatetest282.json");
        fn random_statetest51("tests/GeneralStateTests/stRandom/randomStatetest51.json");
        fn random_statetest144("tests/GeneralStateTests/stRandom/randomStatetest144.json");
        fn random_statetest92("tests/GeneralStateTests/stRandom/randomStatetest92.json");
        fn random_statetest304("tests/GeneralStateTests/stRandom/randomStatetest304.json");
        fn random_statetest187("tests/GeneralStateTests/stRandom/randomStatetest187.json");
        fn random_statetest216("tests/GeneralStateTests/stRandom/randomStatetest216.json");
        fn random_statetest353("tests/GeneralStateTests/stRandom/randomStatetest353.json");
        fn random_statetest200("tests/GeneralStateTests/stRandom/randomStatetest200.json");
        fn random_statetest345("tests/GeneralStateTests/stRandom/randomStatetest345.json");
        fn random_statetest129("tests/GeneralStateTests/stRandom/randomStatetest129.json");
        fn random_statetest84("tests/GeneralStateTests/stRandom/randomStatetest84.json");
        fn random_statetest257("tests/GeneralStateTests/stRandom/randomStatetest257.json");
        fn random_statetest312("tests/GeneralStateTests/stRandom/randomStatetest312.json");
        fn random_statetest191("tests/GeneralStateTests/stRandom/randomStatetest191.json");
        fn random_statetest190("tests/GeneralStateTests/stRandom/randomStatetest190.json");
        fn random_statetest313("tests/GeneralStateTests/stRandom/randomStatetest313.json");
        fn random_statetest201("tests/GeneralStateTests/stRandom/randomStatetest201.json");
        fn random_statetest217("tests/GeneralStateTests/stRandom/randomStatetest217.json");
        fn random_statetest352("tests/GeneralStateTests/stRandom/randomStatetest352.json");
        fn random_statetest169("tests/GeneralStateTests/stRandom/randomStatetest169.json");
        fn random_statetest305("tests/GeneralStateTests/stRandom/randomStatetest305.json");
        fn random_statetest112("tests/GeneralStateTests/stRandom/randomStatetest112.json");
        fn random_statetest145("tests/GeneralStateTests/stRandom/randomStatetest145.json");
        fn random_statetest283("tests/GeneralStateTests/stRandom/randomStatetest283.json");
        fn random_statetest329("tests/GeneralStateTests/stRandom/randomStatetest329.json");
        fn random_statetest153("tests/GeneralStateTests/stRandom/randomStatetest153.json");
        fn random_statetest295("tests/GeneralStateTests/stRandom/randomStatetest295.json");
        fn random_statetest104("tests/GeneralStateTests/stRandom/randomStatetest104.json");
        fn random_statetest11("tests/GeneralStateTests/stRandom/randomStatetest11.json");
        fn random_statetest368("tests/GeneralStateTests/stRandom/randomStatetest368.json");
        fn random_statetest89("tests/GeneralStateTests/stRandom/randomStatetest89.json");
        fn random_statetest66("tests/GeneralStateTests/stRandom/randomStatetest66.json");
        fn random_statetest173("tests/GeneralStateTests/stRandom/randomStatetest173.json");
        fn random_statetest31("tests/GeneralStateTests/stRandom/randomStatetest31.json");
        fn random_statetest348("tests/GeneralStateTests/stRandom/randomStatetest348.json");
        fn random_statetest124("tests/GeneralStateTests/stRandom/randomStatetest124.json");
        fn random_statetest27("tests/GeneralStateTests/stRandom/randomStatetest27.json");
        fn random_statetest309("tests/GeneralStateTests/stRandom/randomStatetest309.json");
        fn random_statetest372("tests/GeneralStateTests/stRandom/randomStatetest372.json");
        fn random_statetest237("tests/GeneralStateTests/stRandom/randomStatetest237.json");
        fn random_statetest325("tests/GeneralStateTests/stRandom/randomStatetest325.json");
        fn random_statetest260("tests/GeneralStateTests/stRandom/randomStatetest260.json");
        fn random_statetest149("tests/GeneralStateTests/stRandom/randomStatetest149.json");
        fn random_statetest299("tests/GeneralStateTests/stRandom/randomStatetest299.json");
        fn random_statetest333("tests/GeneralStateTests/stRandom/randomStatetest333.json");
        fn random_statetest276("tests/GeneralStateTests/stRandom/randomStatetest276.json");
        fn random_statetest364("tests/GeneralStateTests/stRandom/randomStatetest364.json");
        fn random_statetest221("tests/GeneralStateTests/stRandom/randomStatetest221.json");
        fn random_statetest108("tests/GeneralStateTests/stRandom/randomStatetest108.json");
        fn random_statetest259("tests/GeneralStateTests/stRandom/randomStatetest259.json");
        fn random_statetest24("tests/GeneralStateTests/stRandom/randomStatetest24.json");
        fn random_statetest131("tests/GeneralStateTests/stRandom/randomStatetest131.json");
        fn random_statetest73("tests/GeneralStateTests/stRandom/randomStatetest73.json");
        fn random_statetest166("tests/GeneralStateTests/stRandom/randomStatetest166.json");
        fn random_statetest189("tests/GeneralStateTests/stRandom/randomStatetest189.json");
        fn random_statetest371("tests/GeneralStateTests/stRandom/randomStatetest371.json");
        fn random_statetest263("tests/GeneralStateTests/stRandom/randomStatetest263.json");
        fn random_statetest326("tests/GeneralStateTests/stRandom/randomStatetest326.json");
        fn random_statetest275("tests/GeneralStateTests/stRandom/randomStatetest275.json");
        fn random_statetest49("tests/GeneralStateTests/stRandom/randomStatetest49.json");
        fn random_statetest222("tests/GeneralStateTests/stRandom/randomStatetest222.json");
        fn random_statetest367("tests/GeneralStateTests/stRandom/randomStatetest367.json");
        fn random_statetest4("tests/GeneralStateTests/stRandom/randomStatetest4.json");
        fn random_statetest69("tests/GeneralStateTests/stRandom/randomStatetest69.json");
        fn random_statetest310("tests/GeneralStateTests/stRandom/randomStatetest310.json");
        fn random_statetest347("tests/GeneralStateTests/stRandom/randomStatetest347.json");
        fn random_statetest202("tests/GeneralStateTests/stRandom/randomStatetest202.json");
        fn random_statetest28("tests/GeneralStateTests/stRandom/randomStatetest28.json");
        fn random_statetest351("tests/GeneralStateTests/stRandom/randomStatetest351.json");
        fn random_statetest214("tests/GeneralStateTests/stRandom/randomStatetest214.json");
        fn random_statetest185("tests/GeneralStateTests/stRandom/randomStatetest185.json");
        fn random_statetest90("tests/GeneralStateTests/stRandom/randomStatetest90.json");
        fn random_statetest243("tests/GeneralStateTests/stRandom/randomStatetest243.json");
        fn random_statetest111("tests/GeneralStateTests/stRandom/randomStatetest111.json");
        fn random_statetest238("tests/GeneralStateTests/stRandom/randomStatetest238.json");
        fn random_statetest280("tests/GeneralStateTests/stRandom/randomStatetest280.json");
        fn random_statetest279("tests/GeneralStateTests/stRandom/randomStatetest279.json");
        fn random_statetest296("tests/GeneralStateTests/stRandom/randomStatetest296.json");
        fn random_statetest45("tests/GeneralStateTests/stRandom/randomStatetest45.json");
        fn random_statetest107("tests/GeneralStateTests/stRandom/randomStatetest107.json");
        fn random_statetest384("tests/GeneralStateTests/stRandom/randomStatetest384.json");
        fn random_statetest12("tests/GeneralStateTests/stRandom/randomStatetest12.json");
        fn random_statetest9("tests/GeneralStateTests/stRandom/randomStatetest9.json");
        fn random_statetest13("tests/GeneralStateTests/stRandom/randomStatetest13.json");
        fn random_statetest106("tests/GeneralStateTests/stRandom/randomStatetest106.json");
        fn random_statetest278("tests/GeneralStateTests/stRandom/randomStatetest278.json");
        fn random_statetest297("tests/GeneralStateTests/stRandom/randomStatetest297.json");
        fn random_statetest151("tests/GeneralStateTests/stRandom/randomStatetest151.json");
        fn random_statetest281("tests/GeneralStateTests/stRandom/randomStatetest281.json");
        fn random_statetest52("tests/GeneralStateTests/stRandom/randomStatetest52.json");
        fn random_statetest147("tests/GeneralStateTests/stRandom/randomStatetest147.json");
        fn random_statetest110("tests/GeneralStateTests/stRandom/randomStatetest110.json");
        fn random_statetest242("tests/GeneralStateTests/stRandom/randomStatetest242.json");
        fn random_statetest184("tests/GeneralStateTests/stRandom/randomStatetest184.json");
        fn random_statetest29("tests/GeneralStateTests/stRandom/randomStatetest29.json");
        fn random_statetest350("tests/GeneralStateTests/stRandom/randomStatetest350.json");
        fn random_statetest215("tests/GeneralStateTests/stRandom/randomStatetest215.json");
        fn random_statetest346("tests/GeneralStateTests/stRandom/randomStatetest346.json");
        fn random_statetest311("tests/GeneralStateTests/stRandom/randomStatetest311.json");
        fn random_statetest87("tests/GeneralStateTests/stRandom/randomStatetest87.json");
        fn random_statetest254("tests/GeneralStateTests/stRandom/randomStatetest254.json");
        fn random_statetest192("tests/GeneralStateTests/stRandom/randomStatetest192.json");
        fn random_statetest366("tests/GeneralStateTests/stRandom/randomStatetest366.json");
        fn random_statetest5("tests/GeneralStateTests/stRandom/randomStatetest5.json");
        fn random_statetest274("tests/GeneralStateTests/stRandom/randomStatetest274.json");
        fn random_statetest327("tests/GeneralStateTests/stRandom/randomStatetest327.json");
        fn random_statetest370("tests/GeneralStateTests/stRandom/randomStatetest370.json");
        fn random_statetest167("tests/GeneralStateTests/stRandom/randomStatetest167.json");
        fn random_statetest188("tests/GeneralStateTests/stRandom/randomStatetest188.json");
        fn random_statetest72("tests/GeneralStateTests/stRandom/randomStatetest72.json");
        fn random_statetest130("tests/GeneralStateTests/stRandom/randomStatetest130.json");
        fn random_statetest25("tests/GeneralStateTests/stRandom/randomStatetest25.json");
        fn random_statetest219("tests/GeneralStateTests/stRandom/randomStatetest219.json");
        fn random_statetest126("tests/GeneralStateTests/stRandom/randomStatetest126.json");
        fn random_statetest33("tests/GeneralStateTests/stRandom/randomStatetest33.json");
        fn random_statetest171("tests/GeneralStateTests/stRandom/randomStatetest171.json");
        fn random_statetest64("tests/GeneralStateTests/stRandom/randomStatetest64.json");
        fn random_statetest195("tests/GeneralStateTests/stRandom/randomStatetest195.json");
        fn random_statetest316("tests/GeneralStateTests/stRandom/randomStatetest316.json");
        fn random_statetest80("tests/GeneralStateTests/stRandom/randomStatetest80.json");
        fn random_statetest204("tests/GeneralStateTests/stRandom/randomStatetest204.json");
        fn random_statetest357("tests/GeneralStateTests/stRandom/randomStatetest357.json");
        fn random_statetest212("tests/GeneralStateTests/stRandom/randomStatetest212.json");
        fn random_statetest183("tests/GeneralStateTests/stRandom/randomStatetest183.json");
        fn random_statetest300("tests/GeneralStateTests/stRandom/randomStatetest300.json");
        fn random_statetest96("tests/GeneralStateTests/stRandom/randomStatetest96.json");
        fn random_statetest245("tests/GeneralStateTests/stRandom/randomStatetest245.json");
        fn random_statetest117("tests/GeneralStateTests/stRandom/randomStatetest117.json");
        fn random_statetest269("tests/GeneralStateTests/stRandom/randomStatetest269.json");
        fn random_statetest55("tests/GeneralStateTests/stRandom/randomStatetest55.json");
        fn random_statetest286("tests/GeneralStateTests/stRandom/randomStatetest286.json");
        fn random_statetest156("tests/GeneralStateTests/stRandom/randomStatetest156.json");
        fn random_statetest290("tests/GeneralStateTests/stRandom/randomStatetest290.json");
        fn random_statetest43("tests/GeneralStateTests/stRandom/randomStatetest43.json");
        fn random_statetest382("tests/GeneralStateTests/stRandom/randomStatetest382.json");
        fn random_statetest228("tests/GeneralStateTests/stRandom/randomStatetest228.json");
        fn random_statetest14("tests/GeneralStateTests/stRandom/randomStatetest14.json");
        fn random_statetest63("tests/GeneralStateTests/stRandom/randomStatetest63.json");
        fn random_statetest176("tests/GeneralStateTests/stRandom/randomStatetest176.json");
        fn random_statetest199("tests/GeneralStateTests/stRandom/randomStatetest199.json");
        fn random_statetest208("tests/GeneralStateTests/stRandom/randomStatetest208.json");
        fn random_statetest121("tests/GeneralStateTests/stRandom/randomStatetest121.json");
        fn random_statetest22("tests/GeneralStateTests/stRandom/randomStatetest22.json");
        fn random_statetest137("tests/GeneralStateTests/stRandom/randomStatetest137.json");
        fn random_statetest75("tests/GeneralStateTests/stRandom/randomStatetest75.json");
        fn random_statetest249("tests/GeneralStateTests/stRandom/randomStatetest249.json");
        fn random_statetest232("tests/GeneralStateTests/stRandom/randomStatetest232.json");
        fn random_statetest265("tests/GeneralStateTests/stRandom/randomStatetest265.json");
        fn random_statetest320("tests/GeneralStateTests/stRandom/randomStatetest320.json");
        fn random_statetest59("tests/GeneralStateTests/stRandom/randomStatetest59.json");
        fn random_statetest273("tests/GeneralStateTests/stRandom/randomStatetest273.json");
        fn random_statetest336("tests/GeneralStateTests/stRandom/randomStatetest336.json");
        fn random_statetest2("tests/GeneralStateTests/stRandom/randomStatetest2.json");
        fn random_statetest361("tests/GeneralStateTests/stRandom/randomStatetest361.json");
        fn random_statetest18("tests/GeneralStateTests/stRandom/randomStatetest18.json");
        fn random_statetest225("tests/GeneralStateTests/stRandom/randomStatetest225.json");
        fn random_statetest3("tests/GeneralStateTests/stRandom/randomStatetest3.json");
        fn random_statetest360("tests/GeneralStateTests/stRandom/randomStatetest360.json");
        fn random_statetest19("tests/GeneralStateTests/stRandom/randomStatetest19.json");
        fn random_statetest337("tests/GeneralStateTests/stRandom/randomStatetest337.json");
        fn random_statetest264("tests/GeneralStateTests/stRandom/randomStatetest264.json");
        fn random_statetest321("tests/GeneralStateTests/stRandom/randomStatetest321.json");
        fn random_statetest58("tests/GeneralStateTests/stRandom/randomStatetest58.json");
        fn random_statetest233("tests/GeneralStateTests/stRandom/randomStatetest233.json");
        fn random_statetest376("tests/GeneralStateTests/stRandom/randomStatetest376.json");
        fn random_statetest161("tests/GeneralStateTests/stRandom/randomStatetest161.json");
        fn random_statetest74("tests/GeneralStateTests/stRandom/randomStatetest74.json");
        fn random_statetest23("tests/GeneralStateTests/stRandom/randomStatetest23.json");
        fn random_statetest120("tests/GeneralStateTests/stRandom/randomStatetest120.json");
        fn random_statetest209("tests/GeneralStateTests/stRandom/randomStatetest209.json");
        fn random_statetest177("tests/GeneralStateTests/stRandom/randomStatetest177.json");
        fn random_statetest198("tests/GeneralStateTests/stRandom/randomStatetest198.json");
        fn random_statetest62("tests/GeneralStateTests/stRandom/randomStatetest62.json");
        fn random_statetest383("tests/GeneralStateTests/stRandom/randomStatetest383.json");
        fn random_statetest15("tests/GeneralStateTests/stRandom/randomStatetest15.json");
        fn random_statetest42("tests/GeneralStateTests/stRandom/randomStatetest42.json");
        fn random_statetest291("tests/GeneralStateTests/stRandom/randomStatetest291.json");
        fn random_statetest157("tests/GeneralStateTests/stRandom/randomStatetest157.json");
        fn random_statetest268("tests/GeneralStateTests/stRandom/randomStatetest268.json");
        fn random_statetest287("tests/GeneralStateTests/stRandom/randomStatetest287.json");
        fn random_statetest54("tests/GeneralStateTests/stRandom/randomStatetest54.json");
        fn random_statetest116("tests/GeneralStateTests/stRandom/randomStatetest116.json");
        fn random_statetest301("tests/GeneralStateTests/stRandom/randomStatetest301.json");
        fn random_statetest78("tests/GeneralStateTests/stRandom/randomStatetest78.json");
        fn random_statetest244("tests/GeneralStateTests/stRandom/randomStatetest244.json");
        fn random_statetest97("tests/GeneralStateTests/stRandom/randomStatetest97.json");
        fn random_statetest356("tests/GeneralStateTests/stRandom/randomStatetest356.json");
        fn random_statetest340("tests/GeneralStateTests/stRandom/randomStatetest340.json");
        fn random_statetest39("tests/GeneralStateTests/stRandom/randomStatetest39.json");
        fn random_statetest81("tests/GeneralStateTests/stRandom/randomStatetest81.json");
        fn random_statetest252("tests/GeneralStateTests/stRandom/randomStatetest252.json");
        fn random_statetest194("tests/GeneralStateTests/stRandom/randomStatetest194.json");
    }
}

mod shanghai {
    define_tests! {

        // --- ALL PASS ---
        fn st_e_i_p3860_limitmeterinitcode_create_init_code_size_limit("tests/GeneralStateTests/Shanghai/stEIP3860-limitmeterinitcode/createInitCodeSizeLimit.json");
        fn st_e_i_p3651_warmcoinbase_coinbase_warm_account_call_gas_fail("tests/GeneralStateTests/Shanghai/stEIP3651-warmcoinbase/coinbaseWarmAccountCallGasFail.json");
        fn st_e_i_p3651_warmcoinbase_coinbase_warm_account_call_gas("tests/GeneralStateTests/Shanghai/stEIP3651-warmcoinbase/coinbaseWarmAccountCallGas.json");
        fn st_e_i_p3855_push0_push0_gas2("tests/GeneralStateTests/Shanghai/stEIP3855-push0/push0Gas2.json");
        fn st_e_i_p3855_push0_push0("tests/GeneralStateTests/Shanghai/stEIP3855-push0/push0.json");
        fn st_e_i_p3855_push0_push0_gas("tests/GeneralStateTests/Shanghai/stEIP3855-push0/push0Gas.json");
        fn st_e_i_p3860_limitmeterinitcode_create2_init_code_size_limit("tests/GeneralStateTests/Shanghai/stEIP3860-limitmeterinitcode/create2InitCodeSizeLimit.json");
        fn st_e_i_p3860_limitmeterinitcode_creation_tx_init_code_size_limit("tests/GeneralStateTests/Shanghai/stEIP3860-limitmeterinitcode/creationTxInitCodeSizeLimit.json");
    }
}

mod st_revert {
    define_tests! {

        // --- ALL PASS (one test fails in original REVM) ---
        fn revert_opcode_multiple_sub_calls("tests/GeneralStateTests/stRevertTest/RevertOpcodeMultipleSubCalls.json");
        fn loop_calls_depth_then_revert3("tests/GeneralStateTests/stRevertTest/LoopCallsDepthThenRevert3.json");
        fn revert_prefound("tests/GeneralStateTests/stRevertTest/RevertPrefound.json");
        fn revert_prefound_call("tests/GeneralStateTests/stRevertTest/RevertPrefoundCall.json");
        fn loop_calls_depth_then_revert2("tests/GeneralStateTests/stRevertTest/LoopCallsDepthThenRevert2.json");
        fn revert_opcode_in_init("tests/GeneralStateTests/stRevertTest/RevertOpcodeInInit.json");
        fn revert_in_delegate_call("tests/GeneralStateTests/stRevertTest/RevertInDelegateCall.json");
        fn revert_remote_sub_call_storage_o_o_g("tests/GeneralStateTests/stRevertTest/RevertRemoteSubCallStorageOOG.json");
        fn revert_opcode_create("tests/GeneralStateTests/stRevertTest/RevertOpcodeCreate.json");
        fn revert_in_call_code("tests/GeneralStateTests/stRevertTest/RevertInCallCode.json");
        fn touch_to_empty_account_revert3_paris("tests/GeneralStateTests/stRevertTest/TouchToEmptyAccountRevert3_Paris.json");
        fn touch_to_empty_account_revert("tests/GeneralStateTests/stRevertTest/TouchToEmptyAccountRevert.json");
        fn state_revert("tests/GeneralStateTests/stRevertTest/stateRevert.json");
        // fn revert_in_create_in_init_paris("tests/GeneralStateTests/stRevertTest/RevertInCreateInInit_Paris.json");
        fn loop_calls_depth_then_revert("tests/GeneralStateTests/stRevertTest/LoopCallsDepthThenRevert.json");
        fn revert_depth_create_address_collision("tests/GeneralStateTests/stRevertTest/RevertDepthCreateAddressCollision.json");
        fn revert_sub_call_storage_o_o_g("tests/GeneralStateTests/stRevertTest/RevertSubCallStorageOOG.json");
        fn revert_precompiled_touch_storage("tests/GeneralStateTests/stRevertTest/RevertPrecompiledTouch_storage.json");
        fn revert_opcode("tests/GeneralStateTests/stRevertTest/RevertOpcode.json");
        fn cost_revert("tests/GeneralStateTests/stRevertTest/costRevert.json");
        fn loop_delegate_calls_depth_then_revert("tests/GeneralStateTests/stRevertTest/LoopDelegateCallsDepthThenRevert.json");
        fn revert_opcode_with_big_output_in_init("tests/GeneralStateTests/stRevertTest/RevertOpcodeWithBigOutputInInit.json");
        fn revert_prefound_call_o_o_g("tests/GeneralStateTests/stRevertTest/RevertPrefoundCallOOG.json");
        fn revert_on_empty_stack("tests/GeneralStateTests/stRevertTest/RevertOnEmptyStack.json");
        fn python_revert_test_tue201814_1430("tests/GeneralStateTests/stRevertTest/PythonRevertTestTue201814-1430.json");
        fn revert_in_static_call("tests/GeneralStateTests/stRevertTest/RevertInStaticCall.json");
        fn revert_precompiled_touch_noncestorage("tests/GeneralStateTests/stRevertTest/RevertPrecompiledTouch_noncestorage.json");
        fn revert_prefound_empty_o_o_g_paris("tests/GeneralStateTests/stRevertTest/RevertPrefoundEmptyOOG_Paris.json");
        fn revert_prefound_empty_call_o_o_g_paris("tests/GeneralStateTests/stRevertTest/RevertPrefoundEmptyCallOOG_Paris.json");
        fn revert_precompiled_touch_paris("tests/GeneralStateTests/stRevertTest/RevertPrecompiledTouch_Paris.json");
        fn revert_prefound_empty("tests/GeneralStateTests/stRevertTest/RevertPrefoundEmpty.json");
        fn revert_opcode_return("tests/GeneralStateTests/stRevertTest/RevertOpcodeReturn.json");
        fn revert_depth_create_o_o_g("tests/GeneralStateTests/stRevertTest/RevertDepthCreateOOG.json");
        fn revert_precompiled_touch_nonce("tests/GeneralStateTests/stRevertTest/RevertPrecompiledTouch_nonce.json");
        fn touch_to_empty_account_revert2_paris("tests/GeneralStateTests/stRevertTest/TouchToEmptyAccountRevert2_Paris.json");
        fn touch_to_empty_account_revert2("tests/GeneralStateTests/stRevertTest/TouchToEmptyAccountRevert2.json");
        fn revert_sub_call_storage_o_o_g2("tests/GeneralStateTests/stRevertTest/RevertSubCallStorageOOG2.json");
        fn touch_to_empty_account_revert_paris("tests/GeneralStateTests/stRevertTest/TouchToEmptyAccountRevert_Paris.json");
        fn revert_prefound_empty_paris("tests/GeneralStateTests/stRevertTest/RevertPrefoundEmpty_Paris.json");
        fn revert_depth2("tests/GeneralStateTests/stRevertTest/RevertDepth2.json");
        fn touch_to_empty_account_revert3("tests/GeneralStateTests/stRevertTest/TouchToEmptyAccountRevert3.json");
        fn revert_opcode_direct_call("tests/GeneralStateTests/stRevertTest/RevertOpcodeDirectCall.json");
        fn revert_prefound_empty_o_o_g("tests/GeneralStateTests/stRevertTest/RevertPrefoundEmptyOOG.json");
        fn revert_prefound_o_o_g("tests/GeneralStateTests/stRevertTest/RevertPrefoundOOG.json");
        fn revert_opcode_in_calls_on_non_empty_return_data("tests/GeneralStateTests/stRevertTest/RevertOpcodeInCallsOnNonEmptyReturnData.json");
        fn revert_prefound_empty_call("tests/GeneralStateTests/stRevertTest/RevertPrefoundEmptyCall.json");
        fn nashatyrev_suicide_revert("tests/GeneralStateTests/stRevertTest/NashatyrevSuicideRevert.json");
        fn revert_prefound_empty_call_paris("tests/GeneralStateTests/stRevertTest/RevertPrefoundEmptyCall_Paris.json");
        fn revert_in_create_in_init("tests/GeneralStateTests/stRevertTest/RevertInCreateInInit.json");
        fn revert_precompiled_touch_storage_paris("tests/GeneralStateTests/stRevertTest/RevertPrecompiledTouch_storage_Paris.json");
        fn revert_precompiled_touch_exact_o_o_g("tests/GeneralStateTests/stRevertTest/RevertPrecompiledTouchExactOOG.json");
        fn revert_prefound_empty_call_o_o_g("tests/GeneralStateTests/stRevertTest/RevertPrefoundEmptyCallOOG.json");
        fn revert_precompiled_touch("tests/GeneralStateTests/stRevertTest/RevertPrecompiledTouch.json");
        fn revert_opcode_calls("tests/GeneralStateTests/stRevertTest/RevertOpcodeCalls.json");
        fn loop_calls_then_revert("tests/GeneralStateTests/stRevertTest/LoopCallsThenRevert.json");
        fn revert_opcode_in_create_returns("tests/GeneralStateTests/stRevertTest/RevertOpcodeInCreateReturns.json");
        fn revert_precompiled_touch_exact_o_o_g_paris("tests/GeneralStateTests/stRevertTest/RevertPrecompiledTouchExactOOG_Paris.json");
    }
}

mod st_init_code_test {
    define_tests! {

        // --- ALL PASS ---
        fn transaction_create_auto_suicide_contract("tests/GeneralStateTests/stInitCodeTest/TransactionCreateAutoSuicideContract.json");
        fn transaction_create_stop_in_initcode("tests/GeneralStateTests/stInitCodeTest/TransactionCreateStopInInitcode.json");
        fn call_recursive_contract("tests/GeneralStateTests/stInitCodeTest/CallRecursiveContract.json");
        fn call_contract_to_create_contract_which_would_create_contract_in_init_code("tests/GeneralStateTests/stInitCodeTest/CallContractToCreateContractWhichWouldCreateContractInInitCode.json");
        fn call_contract_to_create_contract_o_o_g_bonus_gas("tests/GeneralStateTests/stInitCodeTest/CallContractToCreateContractOOGBonusGas.json");
        fn return_test2("tests/GeneralStateTests/stInitCodeTest/ReturnTest2.json");
        fn call_contract_to_create_contract_o_o_g("tests/GeneralStateTests/stInitCodeTest/CallContractToCreateContractOOG.json");
        fn return_test("tests/GeneralStateTests/stInitCodeTest/ReturnTest.json");
        fn stack_under_flow_contract_creation("tests/GeneralStateTests/stInitCodeTest/StackUnderFlowContractCreation.json");
        fn out_of_gas_prefunded_contract_creation("tests/GeneralStateTests/stInitCodeTest/OutOfGasPrefundedContractCreation.json");
        fn out_of_gas_contract_creation("tests/GeneralStateTests/stInitCodeTest/OutOfGasContractCreation.json");
        fn transaction_create_suicide_in_initcode("tests/GeneralStateTests/stInitCodeTest/TransactionCreateSuicideInInitcode.json");
        fn call_contract_to_create_contract_which_would_create_contract_if_called("tests/GeneralStateTests/stInitCodeTest/CallContractToCreateContractWhichWouldCreateContractIfCalled.json");
        fn call_contract_to_create_contract_no_cash("tests/GeneralStateTests/stInitCodeTest/CallContractToCreateContractNoCash.json");
        fn call_the_contract_to_create_empty_contract("tests/GeneralStateTests/stInitCodeTest/CallTheContractToCreateEmptyContract.json");
        fn call_contract_to_create_contract_and_call_it_o_o_g("tests/GeneralStateTests/stInitCodeTest/CallContractToCreateContractAndCallItOOG.json");
        fn transaction_create_random_init_code("tests/GeneralStateTests/stInitCodeTest/TransactionCreateRandomInitCode.json");
    }
}

mod st_create_test {
    define_tests! {

        // --- ALL PASS ---
        fn create_transaction_call_data("tests/GeneralStateTests/stCreateTest/CreateTransactionCallData.json");
        fn create_large_result("tests/GeneralStateTests/stCreateTest/createLargeResult.json");
        fn create_o_o_gafter_init_code_returndata("tests/GeneralStateTests/stCreateTest/CreateOOGafterInitCodeReturndata.json");
        fn c_r_e_a_t_e_contract_suicide_during_init_with_value_to_itself("tests/GeneralStateTests/stCreateTest/CREATE_ContractSuicideDuringInit_WithValueToItself.json");
        fn create_collision_to_empty2("tests/GeneralStateTests/stCreateTest/CreateCollisionToEmpty2.json");
        fn create_o_o_gafter_init_code_revert("tests/GeneralStateTests/stCreateTest/CreateOOGafterInitCodeRevert.json");
        fn create_results("tests/GeneralStateTests/stCreateTest/CreateResults.json");
        fn create_collision_to_empty("tests/GeneralStateTests/stCreateTest/CreateCollisionToEmpty.json");
        fn code_in_constructor("tests/GeneralStateTests/stCreateTest/CodeInConstructor.json");
        fn c_r_e_a_t_e_empty_contract_with_balance("tests/GeneralStateTests/stCreateTest/CREATE_EmptyContractWithBalance.json");
        fn c_r_e_a_t_e_contract_s_s_t_o_r_e_during_init("tests/GeneralStateTests/stCreateTest/CREATE_ContractSSTOREDuringInit.json");
        fn c_r_e_a_t_e_contract_suicide_during_init_then_store_then_return("tests/GeneralStateTests/stCreateTest/CREATE_ContractSuicideDuringInit_ThenStoreThenReturn.json");
        fn create_address_warm_after_fail("tests/GeneralStateTests/stCreateTest/CreateAddressWarmAfterFail.json");
        fn create_o_o_gafter_init_code_returndata3("tests/GeneralStateTests/stCreateTest/CreateOOGafterInitCodeReturndata3.json");
        fn create_transaction_refund_e_f("tests/GeneralStateTests/stCreateTest/CreateTransactionRefundEF.json");
        fn c_r_e_a_t_e_high_nonce("tests/GeneralStateTests/stCreateTest/CREATE_HighNonce.json");
        fn c_r_e_a_t_e_empty_contract_with_storage_and_call_it_0wei("tests/GeneralStateTests/stCreateTest/CREATE_EmptyContractWithStorageAndCallIt_0wei.json");
        fn c_r_e_a_t_e_e_contract_create_n_e_contract_in_init_o_o_g_tr("tests/GeneralStateTests/stCreateTest/CREATE_EContractCreateNEContractInInitOOG_Tr.json");
        fn create_o_o_gafter_init_code_returndata2("tests/GeneralStateTests/stCreateTest/CreateOOGafterInitCodeReturndata2.json");
        fn transaction_collision_to_empty("tests/GeneralStateTests/stCreateTest/TransactionCollisionToEmpty.json");
        fn c_r_e_a_t_e_empty_contract_with_storage_and_call_it_1wei("tests/GeneralStateTests/stCreateTest/CREATE_EmptyContractWithStorageAndCallIt_1wei.json");
        fn c_r_e_a_t_e_contract_suicide_during_init_with_value("tests/GeneralStateTests/stCreateTest/CREATE_ContractSuicideDuringInit_WithValue.json");
        fn c_r_e_a_t_e_contract_suicide_during_init("tests/GeneralStateTests/stCreateTest/CREATE_ContractSuicideDuringInit.json");
        fn create_fail_result("tests/GeneralStateTests/stCreateTest/createFailResult.json");
        fn c_r_e_a_t_e_contract_r_e_t_u_r_n_big_offset("tests/GeneralStateTests/stCreateTest/CREATE_ContractRETURNBigOffset.json");
        fn c_r_e_a_t_e_high_nonce_minus1("tests/GeneralStateTests/stCreateTest/CREATE_HighNonceMinus1.json");
        fn c_r_e_a_t_e2_call_data("tests/GeneralStateTests/stCreateTest/CREATE2_CallData.json");
        fn c_r_e_a_t_e_e_contract_create_e_contract_in_init_tr("tests/GeneralStateTests/stCreateTest/CREATE_EContractCreateEContractInInit_Tr.json");
        fn transaction_collision_to_empty2("tests/GeneralStateTests/stCreateTest/TransactionCollisionToEmpty2.json");
        fn create_transaction_high_nonce("tests/GeneralStateTests/stCreateTest/CreateTransactionHighNonce.json");
        fn create_o_o_g_from_call_refunds("tests/GeneralStateTests/stCreateTest/CreateOOGFromCallRefunds.json");
        fn c_r_e_a_t_e_e_contract_then_c_a_l_l_to_non_existent_acc("tests/GeneralStateTests/stCreateTest/CREATE_EContract_ThenCALLToNonExistentAcc.json");
        fn c_r_e_a_t_e_acreate_b_b_suicide_b_store("tests/GeneralStateTests/stCreateTest/CREATE_AcreateB_BSuicide_BStore.json");
        fn c_r_e_a_t_e2_refund_e_f("tests/GeneralStateTests/stCreateTest/CREATE2_RefundEF.json");
        fn c_r_e_a_t_e_e_contract_create_n_e_contract_in_init_tr("tests/GeneralStateTests/stCreateTest/CREATE_EContractCreateNEContractInInit_Tr.json");
        fn c_r_e_a_t_e_empty_contract_and_call_it_0wei("tests/GeneralStateTests/stCreateTest/CREATE_EmptyContractAndCallIt_0wei.json");
        fn create_o_o_g_from_e_o_a_refunds("tests/GeneralStateTests/stCreateTest/CreateOOGFromEOARefunds.json");
        fn create_o_o_gafter_init_code("tests/GeneralStateTests/stCreateTest/CreateOOGafterInitCode.json");
        fn c_r_e_a_t_e_empty_contract("tests/GeneralStateTests/stCreateTest/CREATE_EmptyContract.json");
        fn c_r_e_a_t_e_first_byte_loop("tests/GeneralStateTests/stCreateTest/CREATE_FirstByte_loop.json");
        fn c_r_e_a_t_e_empty_contract_and_call_it_1wei("tests/GeneralStateTests/stCreateTest/CREATE_EmptyContractAndCallIt_1wei.json");
        fn c_r_e_a_t_e_empty_contract_with_storage("tests/GeneralStateTests/stCreateTest/CREATE_EmptyContractWithStorage.json");
        fn transaction_collision_to_empty_but_code("tests/GeneralStateTests/stCreateTest/TransactionCollisionToEmptyButCode.json");
        fn transaction_collision_to_empty_but_nonce("tests/GeneralStateTests/stCreateTest/TransactionCollisionToEmptyButNonce.json");
        fn c_r_e_a_t_e_empty000_createin_init_code_transaction("tests/GeneralStateTests/stCreateTest/CREATE_empty000CreateinInitCode_Transaction.json");
        fn create_o_o_gafter_init_code_returndata_size("tests/GeneralStateTests/stCreateTest/CreateOOGafterInitCodeReturndataSize.json");
        fn create_o_o_gafter_init_code_revert2("tests/GeneralStateTests/stCreateTest/CreateOOGafterInitCodeRevert2.json");
        fn create_collision_results("tests/GeneralStateTests/stCreateTest/CreateCollisionResults.json");
        fn create_o_o_gafter_max_codesize("tests/GeneralStateTests/stCreateTest/CreateOOGafterMaxCodesize.json");
    }
}

mod st_s_load_test {
    define_tests! {

        // --- ALL PASS ---
        fn sload_gas_cost("tests/GeneralStateTests/stSLoadTest/sloadGasCost.json");
    }
}

mod st_random2 {
    define_tests! {

        // --- ALL PASS ---
        fn random_statetest553("tests/GeneralStateTests/stRandom2/randomStatetest553.json");
        fn random_statetest416("tests/GeneralStateTests/stRandom2/randomStatetest416.json");
        fn random_statetest504("tests/GeneralStateTests/stRandom2/randomStatetest504.json");
        fn random_statetest512("tests/GeneralStateTests/stRandom2/randomStatetest512.json");
        fn random_statetest457("tests/GeneralStateTests/stRandom2/randomStatetest457.json");
        fn random_statetest396("tests/GeneralStateTests/stRandom2/randomStatetest396.json");
        fn random_statetest545("tests/GeneralStateTests/stRandom2/randomStatetest545.json");
        fn random_statetest494("tests/GeneralStateTests/stRandom2/randomStatetest494.json");
        fn random_statetest640("tests/GeneralStateTests/stRandom2/randomStatetest640.json");
        fn random_statetest586("tests/GeneralStateTests/stRandom2/randomStatetest586.json");
        fn random_statetest569("tests/GeneralStateTests/stRandom2/randomStatetest569.json");
        fn random_statetest601("tests/GeneralStateTests/stRandom2/randomStatetest601.json");
        fn random_statetest482("tests/GeneralStateTests/stRandom2/randomStatetest482.json");
        fn random_statetest528("tests/GeneralStateTests/stRandom2/randomStatetest528.json");
        fn random_statetest508("tests/GeneralStateTests/stRandom2/randomStatetest508.json");
        fn random_statetest621("tests/GeneralStateTests/stRandom2/randomStatetest621.json");
        fn random_statetest637("tests/GeneralStateTests/stRandom2/randomStatetest637.json");
        fn random_statetest477("tests/GeneralStateTests/stRandom2/randomStatetest477.json");
        fn random_statetest498("tests/GeneralStateTests/stRandom2/randomStatetest498.json");
        fn random_statetest532("tests/GeneralStateTests/stRandom2/randomStatetest532.json");
        fn random_statetest420("tests/GeneralStateTests/stRandom2/randomStatetest420.json");
        fn random_statetest565("tests/GeneralStateTests/stRandom2/randomStatetest565.json");
        fn random_statetest436("tests/GeneralStateTests/stRandom2/randomStatetest436.json");
        fn random_statetest461("tests/GeneralStateTests/stRandom2/randomStatetest461.json");
        fn random_statetest524("tests/GeneralStateTests/stRandom2/randomStatetest524.json");
        fn random_statetest460("tests/GeneralStateTests/stRandom2/randomStatetest460.json");
        fn random_statetest525("tests/GeneralStateTests/stRandom2/randomStatetest525.json");
        fn random_statetest437("tests/GeneralStateTests/stRandom2/randomStatetest437.json");
        fn random_statetest572("tests/GeneralStateTests/stRandom2/randomStatetest572.json");
        fn random_statetest421("tests/GeneralStateTests/stRandom2/randomStatetest421.json");
        fn random_statetest564("tests/GeneralStateTests/stRandom2/randomStatetest564.json");
        fn random_statetest476("tests/GeneralStateTests/stRandom2/randomStatetest476.json");
        fn random_statetest499("tests/GeneralStateTests/stRandom2/randomStatetest499.json");
        fn random_statetest533("tests/GeneralStateTests/stRandom2/randomStatetest533.json");
        fn random_statetest548("tests/GeneralStateTests/stRandom2/randomStatetest548.json");
        fn random_statetest636("tests/GeneralStateTests/stRandom2/randomStatetest636.json");
        fn random_statetest620("tests/GeneralStateTests/stRandom2/randomStatetest620.json");
        fn random_statetest509("tests/GeneralStateTests/stRandom2/randomStatetest509.json");
        fn random_statetest483("tests/GeneralStateTests/stRandom2/randomStatetest483.json");
        fn random_statetest600("tests/GeneralStateTests/stRandom2/randomStatetest600.json");
        fn random_statetest587("tests/GeneralStateTests/stRandom2/randomStatetest587.json");
        fn random_statetest641("tests/GeneralStateTests/stRandom2/randomStatetest641.json");
        fn random_statetest495("tests/GeneralStateTests/stRandom2/randomStatetest495.json");
        fn random_statetest616("tests/GeneralStateTests/stRandom2/randomStatetest616.json");
        fn random_statetest544("tests/GeneralStateTests/stRandom2/randomStatetest544.json");
        fn random_statetest401("tests/GeneralStateTests/stRandom2/randomStatetest401.json");
        fn random_statetest397("tests/GeneralStateTests/stRandom2/randomStatetest397.json");
        fn random_statetest513("tests/GeneralStateTests/stRandom2/randomStatetest513.json");
        fn random_statetest456("tests/GeneralStateTests/stRandom2/randomStatetest456.json");
        fn random_statetest505("tests/GeneralStateTests/stRandom2/randomStatetest505.json");
        fn random_statetest440("tests/GeneralStateTests/stRandom2/randomStatetest440.json");
        fn random_statetest552("tests/GeneralStateTests/stRandom2/randomStatetest552.json");
        fn random_statetest417("tests/GeneralStateTests/stRandom2/randomStatetest417.json");
        fn random_statetest559("tests/GeneralStateTests/stRandom2/randomStatetest559.json");
        fn random_statetest627("tests/GeneralStateTests/stRandom2/randomStatetest627.json");
        fn random_statetest518("tests/GeneralStateTests/stRandom2/randomStatetest518.json");
        fn random_statetest471("tests/GeneralStateTests/stRandom2/randomStatetest471.json");
        fn random_statetest534("tests/GeneralStateTests/stRandom2/randomStatetest534.json");
        fn random_statetest426("tests/GeneralStateTests/stRandom2/randomStatetest426.json");
        fn random_statetest563("tests/GeneralStateTests/stRandom2/randomStatetest563.json");
        fn random_statetest430("tests/GeneralStateTests/stRandom2/randomStatetest430.json");
        fn random_statetest575("tests/GeneralStateTests/stRandom2/randomStatetest575.json");
        fn random_statetest467("tests/GeneralStateTests/stRandom2/randomStatetest467.json");
        fn random_statetest488("tests/GeneralStateTests/stRandom2/randomStatetest488.json");
        fn random_statetest386("tests/GeneralStateTests/stRandom2/randomStatetest386.json");
        fn random_statetest555("tests/GeneralStateTests/stRandom2/randomStatetest555.json");
        fn random_statetest410("tests/GeneralStateTests/stRandom2/randomStatetest410.json");
        fn random_statetest502("tests/GeneralStateTests/stRandom2/randomStatetest502.json");
        fn random_statetest447("tests/GeneralStateTests/stRandom2/randomStatetest447.json");
        fn random_statetest514("tests/GeneralStateTests/stRandom2/randomStatetest514.json");
        fn random_statetest451("tests/GeneralStateTests/stRandom2/randomStatetest451.json");
        fn random_statetest543("tests/GeneralStateTests/stRandom2/randomStatetest543.json");
        fn random_statetest406("tests/GeneralStateTests/stRandom2/randomStatetest406.json");
        fn random_statetest611("tests/GeneralStateTests/stRandom2/randomStatetest611.json");
        fn random_statetest646("tests/GeneralStateTests/stRandom2/randomStatetest646.json");
        fn random_statetest580("tests/GeneralStateTests/stRandom2/randomStatetest580.json");
        fn random_statetest650("tests/GeneralStateTests/stRandom2/randomStatetest650.json");
        fn random_statetest596("tests/GeneralStateTests/stRandom2/randomStatetest596.json");
        fn random_statetest579("tests/GeneralStateTests/stRandom2/randomStatetest579.json");
        fn random_statetest607("tests/GeneralStateTests/stRandom2/randomStatetest607.json");
        fn random_statetest484("tests/GeneralStateTests/stRandom2/randomStatetest484.json");
        fn random_statetest485("tests/GeneralStateTests/stRandom2/randomStatetest485.json");
        fn random_statetest597("tests/GeneralStateTests/stRandom2/randomStatetest597.json");
        fn random_statetest578("tests/GeneralStateTests/stRandom2/randomStatetest578.json");
        fn random_statetest581("tests/GeneralStateTests/stRandom2/randomStatetest581.json");
        fn random_statetest647("tests/GeneralStateTests/stRandom2/randomStatetest647.json");
        fn random_statetest539("tests/GeneralStateTests/stRandom2/randomStatetest539.json");
        fn random_statetest493("tests/GeneralStateTests/stRandom2/randomStatetest493.json");
        fn random_statetest610("tests/GeneralStateTests/stRandom2/randomStatetest610.json");
        fn random_statetest542("tests/GeneralStateTests/stRandom2/randomStatetest542.json");
        fn random_statetest407("tests/GeneralStateTests/stRandom2/randomStatetest407.json");
        fn random_statetest450("tests/GeneralStateTests/stRandom2/randomStatetest450.json");
        fn random_statetest503("tests/GeneralStateTests/stRandom2/randomStatetest503.json");
        fn random_statetest446("tests/GeneralStateTests/stRandom2/randomStatetest446.json");
        fn random_statetest554("tests/GeneralStateTests/stRandom2/randomStatetest554.json");
        fn random_statetest411("tests/GeneralStateTests/stRandom2/randomStatetest411.json");
        fn random_statetest387("tests/GeneralStateTests/stRandom2/randomStatetest387.json");
        fn random_statetest466("tests/GeneralStateTests/stRandom2/randomStatetest466.json");
        fn random_statetest523("tests/GeneralStateTests/stRandom2/randomStatetest523.json");
        fn random_statetest489("tests/GeneralStateTests/stRandom2/randomStatetest489.json");
        fn random_statetest574("tests/GeneralStateTests/stRandom2/randomStatetest574.json");
        fn random_statetest562("tests/GeneralStateTests/stRandom2/randomStatetest562.json");
        fn random_statetest470("tests/GeneralStateTests/stRandom2/randomStatetest470.json");
        fn random_statetest535("tests/GeneralStateTests/stRandom2/randomStatetest535.json");
        fn random_statetest630("tests/GeneralStateTests/stRandom2/randomStatetest630.json");
        fn random_statetest519("tests/GeneralStateTests/stRandom2/randomStatetest519.json");
        fn random_statetest626("tests/GeneralStateTests/stRandom2/randomStatetest626.json");
        fn random_statetest558("tests/GeneralStateTests/stRandom2/randomStatetest558.json");
        fn random_statetest609("tests/GeneralStateTests/stRandom2/randomStatetest609.json");
        fn random_statetest520("tests/GeneralStateTests/stRandom2/randomStatetest520.json");
        fn random_statetest465("tests/GeneralStateTests/stRandom2/randomStatetest465.json");
        fn random_statetest577("tests/GeneralStateTests/stRandom2/randomStatetest577.json");
        fn random_statetest648("tests/GeneralStateTests/stRandom2/randomStatetest648.json");
        fn random_statetest424("tests/GeneralStateTests/stRandom2/randomStatetest424.json");
        fn random_statetest536("tests/GeneralStateTests/stRandom2/randomStatetest536.json");
        fn random_statetest473("tests/GeneralStateTests/stRandom2/randomStatetest473.json");
        fn random_statetest408("tests/GeneralStateTests/stRandom2/randomStatetest408.json");
        fn random_statetest633("tests/GeneralStateTests/stRandom2/randomStatetest633.json");
        fn random_statetest625("tests/GeneralStateTests/stRandom2/randomStatetest625.json");
        fn random_statetest449("tests/GeneralStateTests/stRandom2/randomStatetest449.json");
        fn random_statetest388("tests/GeneralStateTests/stRandom2/randomStatetest388.json");
        fn random_statetest469("tests/GeneralStateTests/stRandom2/randomStatetest469.json");
        fn random_statetest605("tests/GeneralStateTests/stRandom2/randomStatetest605.json");
        fn random_statetest582("tests/GeneralStateTests/stRandom2/randomStatetest582.json");
        fn random_statetest428("tests/GeneralStateTests/stRandom2/randomStatetest428.json");
        fn random_statetest644("tests/GeneralStateTests/stRandom2/randomStatetest644.json");
        fn random_statetest404("tests/GeneralStateTests/stRandom2/randomStatetest404.json");
        fn random_statetest541("tests/GeneralStateTests/stRandom2/randomStatetest541.json");
        fn random_statetest516("tests/GeneralStateTests/stRandom2/randomStatetest516.json");
        fn random_statetest445("tests/GeneralStateTests/stRandom2/randomStatetest445.json");
        fn random_statetest500("tests/GeneralStateTests/stRandom2/randomStatetest500.json");
        fn random_statetest629("tests/GeneralStateTests/stRandom2/randomStatetest629.json");
        fn random_statetest412("tests/GeneralStateTests/stRandom2/randomStatetest412.json");
        fn random_statetest384("tests/GeneralStateTests/stRandom2/randomStatetest384.json");
        fn random_statetest385("tests/GeneralStateTests/stRandom2/randomStatetest385.json");
        fn random_statetest413("tests/GeneralStateTests/stRandom2/randomStatetest413.json");
        fn random_statetest556("tests/GeneralStateTests/stRandom2/randomStatetest556.json");
        fn random_statetest628("tests/GeneralStateTests/stRandom2/randomStatetest628.json");
        fn random_statetest444("tests/GeneralStateTests/stRandom2/randomStatetest444.json");
        fn random_statetest501("tests/GeneralStateTests/stRandom2/randomStatetest501.json");
        fn random_statetest452("tests/GeneralStateTests/stRandom2/randomStatetest452.json");
        fn random_statetest517("tests/GeneralStateTests/stRandom2/randomStatetest517.json");
        fn random_statetest393("tests/GeneralStateTests/stRandom2/randomStatetest393.json");
        fn random_statetest405("tests/GeneralStateTests/stRandom2/randomStatetest405.json");
        fn random_statetest612("tests/GeneralStateTests/stRandom2/randomStatetest612.json");
        fn random_statetest491("tests/GeneralStateTests/stRandom2/randomStatetest491.json");
        fn random_statetest645("tests/GeneralStateTests/stRandom2/randomStatetest645.json");
        fn random_statetest583("tests/GeneralStateTests/stRandom2/randomStatetest583.json");
        fn random_statetest429("tests/GeneralStateTests/stRandom2/randomStatetest429.json");
        fn random_statetest604("tests/GeneralStateTests/stRandom2/randomStatetest604.json");
        fn random_statetest487("tests/GeneralStateTests/stRandom2/randomStatetest487.json");
        fn random_statetest389("tests/GeneralStateTests/stRandom2/randomStatetest389.json");
        fn random_statetest448("tests/GeneralStateTests/stRandom2/randomStatetest448.json");
        fn random_statetest624("tests/GeneralStateTests/stRandom2/randomStatetest624.json");
        fn random_statetest632("tests/GeneralStateTests/stRandom2/randomStatetest632.json");
        fn random_statetest409("tests/GeneralStateTests/stRandom2/randomStatetest409.json");
        fn random_statetest537("tests/GeneralStateTests/stRandom2/randomStatetest537.json");
        fn random_statetest472("tests/GeneralStateTests/stRandom2/randomStatetest472.json");
        fn random_statetest560("tests/GeneralStateTests/stRandom2/randomStatetest560.json");
        fn random_statetest425("tests/GeneralStateTests/stRandom2/randomStatetest425.json");
        fn random_statetest649("tests/GeneralStateTests/stRandom2/randomStatetest649.json");
        fn random_statetest576("tests/GeneralStateTests/stRandom2/randomStatetest576.json");
        fn random_statetest599("tests/GeneralStateTests/stRandom2/randomStatetest599.json");
        fn random_statetest433("tests/GeneralStateTests/stRandom2/randomStatetest433.json");
        fn random_statetest521("tests/GeneralStateTests/stRandom2/randomStatetest521.json");
        fn random_statetest464("tests/GeneralStateTests/stRandom2/randomStatetest464.json");
        fn random_statetest608("tests/GeneralStateTests/stRandom2/randomStatetest608.json");
        fn random_statetest480("tests/GeneralStateTests/stRandom2/randomStatetest480.json");
        fn random_statetest603("tests/GeneralStateTests/stRandom2/randomStatetest603.json");
        fn random_statetest438("tests/GeneralStateTests/stRandom2/randomStatetest438.json");
        fn random_statetest592("tests/GeneralStateTests/stRandom2/randomStatetest592.json");
        fn random_statetest584("tests/GeneralStateTests/stRandom2/randomStatetest584.json");
        fn random_statetest642("tests/GeneralStateTests/stRandom2/randomStatetest642.json");
        fn random_statetest496("tests/GeneralStateTests/stRandom2/randomStatetest496.json");
        fn random_statetest615("tests/GeneralStateTests/stRandom2/randomStatetest615.json");
        fn random_statetest402("tests/GeneralStateTests/stRandom2/randomStatetest402.json");
        fn random_statetest547("tests/GeneralStateTests/stRandom2/randomStatetest547.json");
        fn random_statetest455("tests/GeneralStateTests/stRandom2/randomStatetest455.json");
        fn random_statetest510("tests/GeneralStateTests/stRandom2/randomStatetest510.json");
        fn random_statetest639("tests/GeneralStateTests/stRandom2/randomStatetest639.json");
        fn random_statetest("tests/GeneralStateTests/stRandom2/randomStatetest.json");
        fn random_statetest443("tests/GeneralStateTests/stRandom2/randomStatetest443.json");
        fn random_statetest506("tests/GeneralStateTests/stRandom2/randomStatetest506.json");
        fn random_statetest414("tests/GeneralStateTests/stRandom2/randomStatetest414.json");
        fn random_statetest526("tests/GeneralStateTests/stRandom2/randomStatetest526.json");
        fn random_statetest571("tests/GeneralStateTests/stRandom2/randomStatetest571.json");
        fn random_statetest567("tests/GeneralStateTests/stRandom2/randomStatetest567.json");
        fn random_statetest422("tests/GeneralStateTests/stRandom2/randomStatetest422.json");
        fn random_statetest588("tests/GeneralStateTests/stRandom2/randomStatetest588.json");
        fn random_statetest475("tests/GeneralStateTests/stRandom2/randomStatetest475.json");
        fn random_statetest398("tests/GeneralStateTests/stRandom2/randomStatetest398.json");
        fn random_statetest635("tests/GeneralStateTests/stRandom2/randomStatetest635.json");
        fn random_statetest418("tests/GeneralStateTests/stRandom2/randomStatetest418.json");
        fn random_statetest419("tests/GeneralStateTests/stRandom2/randomStatetest419.json");
        fn random_statetest458("tests/GeneralStateTests/stRandom2/randomStatetest458.json");
        fn random_statetest399("tests/GeneralStateTests/stRandom2/randomStatetest399.json");
        fn random_statetest531("tests/GeneralStateTests/stRandom2/randomStatetest531.json");
        fn random_statetest474("tests/GeneralStateTests/stRandom2/randomStatetest474.json");
        fn random_statetest618("tests/GeneralStateTests/stRandom2/randomStatetest618.json");
        fn random_statetest566("tests/GeneralStateTests/stRandom2/randomStatetest566.json");
        fn random_statetest589("tests/GeneralStateTests/stRandom2/randomStatetest589.json");
        fn random_statetest435("tests/GeneralStateTests/stRandom2/randomStatetest435.json");
        fn random_statetest527("tests/GeneralStateTests/stRandom2/randomStatetest527.json");
        fn random_statetest462("tests/GeneralStateTests/stRandom2/randomStatetest462.json");
        fn random_statetest415("tests/GeneralStateTests/stRandom2/randomStatetest415.json");
        fn random_statetest550("tests/GeneralStateTests/stRandom2/randomStatetest550.json");
        fn random_statetest442("tests/GeneralStateTests/stRandom2/randomStatetest442.json");
        fn random_statetest507("tests/GeneralStateTests/stRandom2/randomStatetest507.json");
        fn random_statetest638("tests/GeneralStateTests/stRandom2/randomStatetest638.json");
        fn random_statetest454("tests/GeneralStateTests/stRandom2/randomStatetest454.json");
        fn random_statetest511("tests/GeneralStateTests/stRandom2/randomStatetest511.json");
        fn random_statetest395("tests/GeneralStateTests/stRandom2/randomStatetest395.json");
        fn random_statetest546("tests/GeneralStateTests/stRandom2/randomStatetest546.json");
        fn random_statetest497("tests/GeneralStateTests/stRandom2/randomStatetest497.json");
        fn random_statetest478("tests/GeneralStateTests/stRandom2/randomStatetest478.json");
        fn random_statetest643("tests/GeneralStateTests/stRandom2/randomStatetest643.json");
        fn random_statetest585("tests/GeneralStateTests/stRandom2/randomStatetest585.json");
        fn random_statetest439("tests/GeneralStateTests/stRandom2/randomStatetest439.json");
        fn random_statetest602("tests/GeneralStateTests/stRandom2/randomStatetest602.json");
        fn random_statetest481("tests/GeneralStateTests/stRandom2/randomStatetest481.json");
    }
}

mod cancun {
    define_tests! {

        // --- MOST PASS --- (2 tests, I'm not sure if we have full blob support)
        fn st_e_i_p4844_blobtransactions_blobhash_list_bounds6("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds6.json");
        fn st_e_i_p4844_blobtransactions_blobhash_list_bounds7("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds7.json");
        fn st_e_i_p4844_blobtransactions_empty_blobhash_list("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/emptyBlobhashList.json");
        fn st_e_i_p4844_blobtransactions_create_blobhash_tx("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/createBlobhashTx.json");
        // fn st_e_i_p4844_blobtransactions_opcode_blobhash_out_of_range("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/opcodeBlobhashOutOfRange.json");
        fn st_e_i_p4844_blobtransactions_blobhash_list_bounds3("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds3.json");
        // fn st_e_i_p4844_blobtransactions_opcode_blobh_bounds("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/opcodeBlobhBounds.json");
        fn st_e_i_p4844_blobtransactions_wrong_blobhash_version("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/wrongBlobhashVersion.json");
        fn st_e_i_p4844_blobtransactions_blobhash_list_bounds4("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds4.json");
        fn st_e_i_p4844_blobtransactions_blobhash_list_bounds5("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds5.json");

        // --- ALL PASS ---
        fn cancun_st_e_i_p5656_m_c_o_p_y_m_c_o_p_y_memory_hash("tests/GeneralStateTests/Cancun/stEIP5656-MCOPY/MCOPY_memory_hash.json");
        fn cancun_st_e_i_p5656_m_c_o_p_y_m_c_o_p_y_copy_cost("tests/GeneralStateTests/Cancun/stEIP5656-MCOPY/MCOPY_copy_cost.json");
        fn cancun_st_e_i_p5656_mcopy_mcopy("tests/GeneralStateTests/Cancun/stEIP5656-MCOPY/MCOPY.json");
        fn cancun_st_e_i_p5656_m_c_o_p_y_m_c_o_p_y_memory_expansion_cost("tests/GeneralStateTests/Cancun/stEIP5656-MCOPY/MCOPY_memory_expansion_cost.json");

        // --- ALL PASS ---
        fn cancun_st_e_i_p1153_transient_storage_21_tstore_cannot_be_dosd_o_o_o("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/21_tstoreCannotBeDosdOOO.json");
        fn cancun_st_e_i_p1153_transient_storage_09_revert_undoes_all("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/09_revertUndoesAll.json");
        fn cancun_st_e_i_p1153_transient_storage_19_oog_undoes_transient_store("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/19_oogUndoesTransientStore.json");
        fn cancun_st_e_i_p1153_transient_storage_18_tload_after_store("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/18_tloadAfterStore.json");
        fn cancun_st_e_i_p1153_transient_storage_12_tload_delegate_call("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/12_tloadDelegateCall.json");
        fn cancun_st_e_i_p1153_transient_storage_17_tstore_gas("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/17_tstoreGas.json");
        fn cancun_st_e_i_p1153_transient_storage_10_revert_undoes_store_after_return("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/10_revertUndoesStoreAfterReturn.json");
        fn cancun_st_e_i_p1153_transient_storage_06_tstore_in_reentrancy_call("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/06_tstoreInReentrancyCall.json");
        fn cancun_st_e_i_p1153_transient_storage_07_tload_after_reentrancy_store("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/07_tloadAfterReentrancyStore.json");
        fn cancun_st_e_i_p1153_transient_storage_14_revert_after_nested_staticcall("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/14_revertAfterNestedStaticcall.json");
        fn cancun_st_e_i_p1153_transient_storage_08_revert_undoes_transient_store("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/08_revertUndoesTransientStore.json");
        fn cancun_st_e_i_p1153_transient_storage_trans_storage_o_k("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/transStorageOK.json");
        fn cancun_st_e_i_p1153_transient_storage_13_tload_static_call("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/13_tloadStaticCall.json");
        fn cancun_st_e_i_p1153_transient_storage_15_tstore_cannot_be_dosd("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/15_tstoreCannotBeDosd.json");
        fn cancun_st_e_i_p1153_transient_storage_11_tstore_delegate_call("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/11_tstoreDelegateCall.json");
        fn cancun_st_e_i_p1153_transient_storage_03_tload_after_store_is0("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/03_tloadAfterStoreIs0.json");
        fn cancun_st_e_i_p1153_transient_storage_trans_storage_reset("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/transStorageReset.json");
        fn cancun_st_e_i_p1153_transient_storage_02_tload_after_tstore("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/02_tloadAfterTstore.json");
        fn cancun_st_e_i_p1153_transient_storage_01_tload_beginning_txn("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/01_tloadBeginningTxn.json");
        fn cancun_st_e_i_p1153_transient_storage_04_tload_after_call("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/04_tloadAfterCall.json");
        fn cancun_st_e_i_p1153_transient_storage_16_tload_gas("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/16_tloadGas.json");
        fn cancun_st_e_i_p1153_transient_storage_20_oog_undoes_transient_store_in_call("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/20_oogUndoesTransientStoreInCall.json");
        fn cancun_st_e_i_p1153_transient_storage_05_tload_reentrancy("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/05_tloadReentrancy.json");

    }
}

mod st_wallet_test {
    define_tests! {

        // --- ALL PASS ---
        fn wallet_kill_to_wallet("tests/GeneralStateTests/stWalletTest/walletKillToWallet.json");
        fn multi_owned_remove_owner_by_non_owner("tests/GeneralStateTests/stWalletTest/multiOwnedRemoveOwnerByNonOwner.json");
        fn multi_owned_remove_owner_my_self("tests/GeneralStateTests/stWalletTest/multiOwnedRemoveOwner_mySelf.json");
        fn wallet_construction("tests/GeneralStateTests/stWalletTest/walletConstruction.json");
        fn multi_owned_change_owner_no_argument("tests/GeneralStateTests/stWalletTest/multiOwnedChangeOwnerNoArgument.json");
        fn day_limit_construction_o_o_g("tests/GeneralStateTests/stWalletTest/dayLimitConstructionOOG.json");
        fn wallet_execute_over_daily_limit_only_one_owner_new("tests/GeneralStateTests/stWalletTest/walletExecuteOverDailyLimitOnlyOneOwnerNew.json");
        fn wallet_change_requirement_remove_pending_transaction("tests/GeneralStateTests/stWalletTest/walletChangeRequirementRemovePendingTransaction.json");
        fn wallet_kill_not_by_owner("tests/GeneralStateTests/stWalletTest/walletKillNotByOwner.json");
        fn multi_owned_change_requirement_to1("tests/GeneralStateTests/stWalletTest/multiOwnedChangeRequirementTo1.json");
        fn multi_owned_add_owner_add_myself("tests/GeneralStateTests/stWalletTest/multiOwnedAddOwnerAddMyself.json");
        fn multi_owned_construction_not_enough_gas("tests/GeneralStateTests/stWalletTest/multiOwnedConstructionNotEnoughGas.json");
        fn wallet_default_with_out_value("tests/GeneralStateTests/stWalletTest/walletDefaultWithOutValue.json");
        fn multi_owned_is_owner_false("tests/GeneralStateTests/stWalletTest/multiOwnedIsOwnerFalse.json");
        fn wallet_construction_o_o_g("tests/GeneralStateTests/stWalletTest/walletConstructionOOG.json");
        fn multi_owned_is_owner_true("tests/GeneralStateTests/stWalletTest/multiOwnedIsOwnerTrue.json");
        fn day_limit_set_daily_limit("tests/GeneralStateTests/stWalletTest/dayLimitSetDailyLimit.json");
        fn day_limit_reset_spent_today("tests/GeneralStateTests/stWalletTest/dayLimitResetSpentToday.json");
        fn multi_owned_change_requirement_to0("tests/GeneralStateTests/stWalletTest/multiOwnedChangeRequirementTo0.json");
        fn day_limit_set_daily_limit_no_data("tests/GeneralStateTests/stWalletTest/dayLimitSetDailyLimitNoData.json");
        fn day_limit_construction_partial("tests/GeneralStateTests/stWalletTest/dayLimitConstructionPartial.json");
        fn multi_owned_remove_owner("tests/GeneralStateTests/stWalletTest/multiOwnedRemoveOwner.json");
        fn multi_owned_construction_not_enough_gas_partial("tests/GeneralStateTests/stWalletTest/multiOwnedConstructionNotEnoughGasPartial.json");
        fn multi_owned_change_requirement_to2("tests/GeneralStateTests/stWalletTest/multiOwnedChangeRequirementTo2.json");
        fn multi_owned_construction_correct("tests/GeneralStateTests/stWalletTest/multiOwnedConstructionCorrect.json");
        fn multi_owned_add_owner("tests/GeneralStateTests/stWalletTest/multiOwnedAddOwner.json");
        fn multi_owned_remove_owner_owner_is_not_owner("tests/GeneralStateTests/stWalletTest/multiOwnedRemoveOwner_ownerIsNotOwner.json");
        fn wallet_construction_partial("tests/GeneralStateTests/stWalletTest/walletConstructionPartial.json");
        fn wallet_remove_owner_remove_pending_transaction("tests/GeneralStateTests/stWalletTest/walletRemoveOwnerRemovePendingTransaction.json");
        fn wallet_change_owner_remove_pending_transaction("tests/GeneralStateTests/stWalletTest/walletChangeOwnerRemovePendingTransaction.json");
        fn multi_owned_revoke_nothing("tests/GeneralStateTests/stWalletTest/multiOwnedRevokeNothing.json");
        fn wallet_default("tests/GeneralStateTests/stWalletTest/walletDefault.json");
        fn wallet_execute_over_daily_limit_multi_owner("tests/GeneralStateTests/stWalletTest/walletExecuteOverDailyLimitMultiOwner.json");
        fn wallet_kill("tests/GeneralStateTests/stWalletTest/walletKill.json");
        fn wallet_execute_under_daily_limit("tests/GeneralStateTests/stWalletTest/walletExecuteUnderDailyLimit.json");
        fn wallet_add_owner_remove_pending_transaction("tests/GeneralStateTests/stWalletTest/walletAddOwnerRemovePendingTransaction.json");
        fn multi_owned_change_owner("tests/GeneralStateTests/stWalletTest/multiOwnedChangeOwner.json");
        fn day_limit_construction("tests/GeneralStateTests/stWalletTest/dayLimitConstruction.json");
        fn wallet_execute_over_daily_limit_only_one_owner("tests/GeneralStateTests/stWalletTest/walletExecuteOverDailyLimitOnlyOneOwner.json");
        fn multi_owned_change_owner_from_not_owner("tests/GeneralStateTests/stWalletTest/multiOwnedChangeOwner_fromNotOwner.json");
        fn wallet_confirm("tests/GeneralStateTests/stWalletTest/walletConfirm.json");
        fn multi_owned_change_owner_to_is_owner("tests/GeneralStateTests/stWalletTest/multiOwnedChangeOwner_toIsOwner.json");
    }
}

mod st_code_copy_test {
    define_tests! {

        // --- ALL PASS ---
        fn ext_code_copy_tests_paris("tests/GeneralStateTests/stCodeCopyTest/ExtCodeCopyTestsParis.json");
        fn ext_code_copy_target_range_longer_than_code_tests("tests/GeneralStateTests/stCodeCopyTest/ExtCodeCopyTargetRangeLongerThanCodeTests.json");
        fn ext_code_copy_tests("tests/GeneralStateTests/stCodeCopyTest/ExtCodeCopyTests.json");
    }
}

mod st_e_i_p2930 {
    define_tests! {

        // --- ALL PASS ---
        fn address_opcodes("tests/GeneralStateTests/stEIP2930/addressOpcodes.json");
        fn manual_create("tests/GeneralStateTests/stEIP2930/manualCreate.json");
        fn storage_costs("tests/GeneralStateTests/stEIP2930/storageCosts.json");
        fn coinbase_t2("tests/GeneralStateTests/stEIP2930/coinbaseT2.json");
        fn coinbase_t01("tests/GeneralStateTests/stEIP2930/coinbaseT01.json");
        fn varied_context("tests/GeneralStateTests/stEIP2930/variedContext.json");
        fn transaction_costs("tests/GeneralStateTests/stEIP2930/transactionCosts.json");
    }
}

mod st_refund_test {
    define_tests! {

        // --- ALL PASS ---
        fn refund_get_ether_back("tests/GeneralStateTests/stRefundTest/refund_getEtherBack.json");
        fn refund_tx_to_suicide("tests/GeneralStateTests/stRefundTest/refund_TxToSuicide.json");
        fn refund50_2("tests/GeneralStateTests/stRefundTest/refund50_2.json");
        fn refund_multimple_suicide("tests/GeneralStateTests/stRefundTest/refund_multimpleSuicide.json");
        fn refund50percent_cap("tests/GeneralStateTests/stRefundTest/refund50percentCap.json");
        fn refund_call_a("tests/GeneralStateTests/stRefundTest/refund_CallA.json");
        fn refund_s_s_t_o_r_e("tests/GeneralStateTests/stRefundTest/refundSSTORE.json");
        fn refund_f_f("tests/GeneralStateTests/stRefundTest/refundFF.json");
        fn refund_call_to_suicide_no_storage("tests/GeneralStateTests/stRefundTest/refund_CallToSuicideNoStorage.json");
        fn refund_reset_frontier("tests/GeneralStateTests/stRefundTest/refundResetFrontier.json");
        fn refund_single_suicide("tests/GeneralStateTests/stRefundTest/refund_singleSuicide.json");
        fn refund_o_o_g("tests/GeneralStateTests/stRefundTest/refund_OOG.json");
        fn refund_call_a_not_enough_gas_in_call("tests/GeneralStateTests/stRefundTest/refund_CallA_notEnoughGasInCall.json");
        fn refund600("tests/GeneralStateTests/stRefundTest/refund600.json");
        fn refund_tx_to_suicide_o_o_g("tests/GeneralStateTests/stRefundTest/refund_TxToSuicideOOG.json");
        fn refund_no_o_o_g_1("tests/GeneralStateTests/stRefundTest/refund_NoOOG_1.json");
        fn refund_call_to_suicide_twice("tests/GeneralStateTests/stRefundTest/refund_CallToSuicideTwice.json");
        fn refund_suicide50procent_cap("tests/GeneralStateTests/stRefundTest/refundSuicide50procentCap.json");
        fn refund_change_non_zero_storage("tests/GeneralStateTests/stRefundTest/refund_changeNonZeroStorage.json");
        fn refund_call_to_suicide_storage("tests/GeneralStateTests/stRefundTest/refund_CallToSuicideStorage.json");
        fn refund_call_a_o_o_g("tests/GeneralStateTests/stRefundTest/refund_CallA_OOG.json");
        fn refund_max("tests/GeneralStateTests/stRefundTest/refundMax.json");
        fn refund50_1("tests/GeneralStateTests/stRefundTest/refund50_1.json");
    }
}

mod st_recursive_create {
    define_tests! {

        // --- ALL PASS ---
        fn recursive_create("tests/GeneralStateTests/stRecursiveCreate/recursiveCreate.json");
        fn recursive_create_return_value("tests/GeneralStateTests/stRecursiveCreate/recursiveCreateReturnValue.json");
    }
}

mod st_pre_compiled_contracts {
    define_tests! {

        // --- ALL PASS ---
        fn blake2_b("tests/GeneralStateTests/stPreCompiledContracts/blake2B.json");
        fn modexp("tests/GeneralStateTests/stPreCompiledContracts/modexp.json");
        fn precomps_e_i_p2929("tests/GeneralStateTests/stPreCompiledContracts/precompsEIP2929.json");
        fn delegatecall09_undefined("tests/GeneralStateTests/stPreCompiledContracts/delegatecall09Undefined.json");
        fn identity_to_bigger("tests/GeneralStateTests/stPreCompiledContracts/identity_to_bigger.json");
        fn identity_to_smaller("tests/GeneralStateTests/stPreCompiledContracts/identity_to_smaller.json");
        fn precomps_e_i_p2929_cancun("tests/GeneralStateTests/stPreCompiledContracts/precompsEIP2929Cancun.json");
        fn modexp_tests("tests/GeneralStateTests/stPreCompiledContracts/modexpTests.json");
        fn sec80("tests/GeneralStateTests/stPreCompiledContracts/sec80.json");
        fn id_precomps("tests/GeneralStateTests/stPreCompiledContracts/idPrecomps.json");
    }
}

mod st_ext_code_hash {
    define_tests! {

        // --- MOST PASS --- (1 test fails on REVM)
        fn ext_code_hash_account_without_code("tests/GeneralStateTests/stExtCodeHash/extCodeHashAccountWithoutCode.json");
        fn ext_code_hash_created_and_deleted_account_call("tests/GeneralStateTests/stExtCodeHash/extCodeHashCreatedAndDeletedAccountCall.json");
        fn extcodehash_empty_paris("tests/GeneralStateTests/stExtCodeHash/extcodehashEmpty_Paris.json");
        fn ext_code_hash_deleted_account2_cancun("tests/GeneralStateTests/stExtCodeHash/extCodeHashDeletedAccount2Cancun.json");
        fn ext_code_hash_deleted_account3("tests/GeneralStateTests/stExtCodeHash/extCodeHashDeletedAccount3.json");
        fn ext_code_hash_subcall_suicide("tests/GeneralStateTests/stExtCodeHash/extCodeHashSubcallSuicide.json");
        fn ext_code_hash_in_init_code("tests/GeneralStateTests/stExtCodeHash/extCodeHashInInitCode.json");
        fn ext_code_hash_deleted_account("tests/GeneralStateTests/stExtCodeHash/extCodeHashDeletedAccount.json");
        fn ext_code_hash_deleted_account2("tests/GeneralStateTests/stExtCodeHash/extCodeHashDeletedAccount2.json");
        fn ext_code_hash_non_existing_account("tests/GeneralStateTests/stExtCodeHash/extCodeHashNonExistingAccount.json");
        fn ext_code_hash_d_e_l_e_g_a_t_e_c_a_l_l("tests/GeneralStateTests/stExtCodeHash/extCodeHashDELEGATECALL.json");
        fn ext_code_copy_bounds("tests/GeneralStateTests/stExtCodeHash/extCodeCopyBounds.json");
        fn ext_code_hash_c_a_l_l("tests/GeneralStateTests/stExtCodeHash/extCodeHashCALL.json");
        fn ext_code_hash_c_a_l_l_c_o_d_e("tests/GeneralStateTests/stExtCodeHash/extCodeHashCALLCODE.json");
        fn ext_code_hash_deleted_account_cancun("tests/GeneralStateTests/stExtCodeHash/extCodeHashDeletedAccountCancun.json");
        fn create_empty_then_extcodehash("tests/GeneralStateTests/stExtCodeHash/createEmptyThenExtcodehash.json");
        fn extcodehash_empty("tests/GeneralStateTests/stExtCodeHash/extcodehashEmpty.json");
        fn ext_code_hash_deleted_account4("tests/GeneralStateTests/stExtCodeHash/extCodeHashDeletedAccount4.json");
        fn ext_code_hash_created_and_deleted_account_recheck_in_outer_call("tests/GeneralStateTests/stExtCodeHash/extCodeHashCreatedAndDeletedAccountRecheckInOuterCall.json");
        fn ext_code_hash_precompiles("tests/GeneralStateTests/stExtCodeHash/extCodeHashPrecompiles.json");
        fn ext_code_hash_dynamic_argument("tests/GeneralStateTests/stExtCodeHash/extCodeHashDynamicArgument.json");
        fn ext_code_hash_self("tests/GeneralStateTests/stExtCodeHash/extCodeHashSelf.json");
        fn ext_code_hash_created_and_deleted_account_static_call("tests/GeneralStateTests/stExtCodeHash/extCodeHashCreatedAndDeletedAccountStaticCall.json");
        fn ext_code_hash_max_code_size("tests/GeneralStateTests/stExtCodeHash/extCodeHashMaxCodeSize.json");
        fn ext_code_hash_created_and_deleted_account("tests/GeneralStateTests/stExtCodeHash/extCodeHashCreatedAndDeletedAccount.json");
        fn ext_code_hash_changed_account("tests/GeneralStateTests/stExtCodeHash/extCodeHashChangedAccount.json");
        fn call_to_suicide_then_extcodehash("tests/GeneralStateTests/stExtCodeHash/callToSuicideThenExtcodehash.json");
        fn ext_code_hash_s_t_a_t_i_c_c_a_l_l("tests/GeneralStateTests/stExtCodeHash/extCodeHashSTATICCALL.json");
        fn code_copy_zero("tests/GeneralStateTests/stExtCodeHash/codeCopyZero.json");
        fn ext_code_hash_deleted_account1("tests/GeneralStateTests/stExtCodeHash/extCodeHashDeletedAccount1.json");
        fn ext_code_hash_new_account("tests/GeneralStateTests/stExtCodeHash/extCodeHashNewAccount.json");
        fn call_to_non_existent("tests/GeneralStateTests/stExtCodeHash/callToNonExistent.json");
        fn code_copy_zero_paris("tests/GeneralStateTests/stExtCodeHash/codeCopyZero_Paris.json");
        fn ext_code_hash_deleted_account1_cancun("tests/GeneralStateTests/stExtCodeHash/extCodeHashDeletedAccount1Cancun.json");
        fn ext_code_hash_self_in_init("tests/GeneralStateTests/stExtCodeHash/extCodeHashSelfInInit.json");
        fn ext_code_hash_subcall_o_o_g("tests/GeneralStateTests/stExtCodeHash/extCodeHashSubcallOOG.json");
        fn dynamic_account_overwrite_empty("tests/GeneralStateTests/stExtCodeHash/dynamicAccountOverwriteEmpty.json");
        // fn dynamic_account_overwrite_empty_paris("tests/GeneralStateTests/stExtCodeHash/dynamicAccountOverwriteEmpty_Paris.json");
        fn ext_code_hash_subcall_suicide_cancun("tests/GeneralStateTests/stExtCodeHash/extCodeHashSubcallSuicideCancun.json");
    }
}

mod st_bugs {
    define_tests! {

        // --- ALL PASS ---
        fn returndatacopy_python_bug_tue_03_48_41_1432("tests/GeneralStateTests/stBugs/returndatacopyPythonBug_Tue_03_48_41-1432.json");
        fn evm_bytecode("tests/GeneralStateTests/stBugs/evmBytecode.json");
        fn random_statetest_d_e_f_a_u_l_t_tue_07_58_41_15153_575192_london("tests/GeneralStateTests/stBugs/randomStatetestDEFAULT-Tue_07_58_41-15153-575192_london.json");
        fn staticcall_createfails("tests/GeneralStateTests/stBugs/staticcall_createfails.json");
        fn random_statetest_d_e_f_a_u_l_t_tue_07_58_41_15153_575192("tests/GeneralStateTests/stBugs/randomStatetestDEFAULT-Tue_07_58_41-15153-575192.json");
    }
}

mod st_example {
    define_tests! {

        // --- ALL PASS ---
        fn ranges_example("tests/GeneralStateTests/stExample/rangesExample.json");
        fn eip1559("tests/GeneralStateTests/stExample/eip1559.json");
        fn yul_example("tests/GeneralStateTests/stExample/yulExample.json");
        fn indexes_omit_example("tests/GeneralStateTests/stExample/indexesOmitExample.json");
        fn labels_example("tests/GeneralStateTests/stExample/labelsExample.json");
        fn invalid_tr("tests/GeneralStateTests/stExample/invalidTr.json");
        fn access_list_example("tests/GeneralStateTests/stExample/accessListExample.json");
        fn add11("tests/GeneralStateTests/stExample/add11.json");
        fn solidity_example("tests/GeneralStateTests/stExample/solidityExample.json");
        fn add11_yml("tests/GeneralStateTests/stExample/add11_yml.json");
        fn basefee_example("tests/GeneralStateTests/stExample/basefeeExample.json");
        fn merge_test("tests/GeneralStateTests/stExample/mergeTest.json");
    }
}

mod st_transition_test {
    define_tests! {

        // --- ALL PASS ---
        fn delegatecall_at_transition("tests/GeneralStateTests/stTransitionTest/delegatecallAtTransition.json");
        fn create_name_registrator_per_txs_before("tests/GeneralStateTests/stTransitionTest/createNameRegistratorPerTxsBefore.json");
        fn delegatecall_after_transition("tests/GeneralStateTests/stTransitionTest/delegatecallAfterTransition.json");
        fn delegatecall_before_transition("tests/GeneralStateTests/stTransitionTest/delegatecallBeforeTransition.json");
        fn create_name_registrator_per_txs_at("tests/GeneralStateTests/stTransitionTest/createNameRegistratorPerTxsAt.json");
        fn create_name_registrator_per_txs_after("tests/GeneralStateTests/stTransitionTest/createNameRegistratorPerTxsAfter.json");
    }
}

mod st_call_codes {
    define_tests! {

        // --- ALL PASS ---
        fn callcallcode_01("tests/GeneralStateTests/stCallCodes/callcallcode_01.json");
        fn callcallcallcode_001("tests/GeneralStateTests/stCallCodes/callcallcallcode_001.json");
        fn callcallcall_000_o_o_g_m_after("tests/GeneralStateTests/stCallCodes/callcallcall_000_OOGMAfter.json");
        fn callcodecallcall_100_o_o_g_e("tests/GeneralStateTests/stCallCodes/callcodecallcall_100_OOGE.json");
        fn callcode_dynamic_code("tests/GeneralStateTests/stCallCodes/callcodeDynamicCode.json");
        fn callcallcallcode_001_suicide_end("tests/GeneralStateTests/stCallCodes/callcallcallcode_001_SuicideEnd.json");
        fn callcodecallcode_11("tests/GeneralStateTests/stCallCodes/callcodecallcode_11.json");
        fn callcodecallcodecall_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallCodes/callcodecallcodecall_ABCB_RECURSIVE.json");
        fn callcodecallcallcode_101("tests/GeneralStateTests/stCallCodes/callcodecallcallcode_101.json");
        fn callcodecallcall_100_suicide_end("tests/GeneralStateTests/stCallCodes/callcodecallcall_100_SuicideEnd.json");
        fn callcallcall_000_suicide_middle("tests/GeneralStateTests/stCallCodes/callcallcall_000_SuicideMiddle.json");
        fn callcodecallcodecallcode_111_o_o_g_m_before("tests/GeneralStateTests/stCallCodes/callcodecallcodecallcode_111_OOGMBefore.json");
        fn callcallcallcode_001_o_o_g_m_after("tests/GeneralStateTests/stCallCodes/callcallcallcode_001_OOGMAfter.json");
        fn callcodecallcodecallcode_111_o_o_g_m_after("tests/GeneralStateTests/stCallCodes/callcodecallcodecallcode_111_OOGMAfter.json");
        fn callcallcall_000_o_o_g_m_before("tests/GeneralStateTests/stCallCodes/callcallcall_000_OOGMBefore.json");
        fn callcodecallcodecall_110_suicide_end("tests/GeneralStateTests/stCallCodes/callcodecallcodecall_110_SuicideEnd.json");
        fn callcallcallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallCodes/callcallcallcode_ABCB_RECURSIVE.json");
        fn callcallcode_01_suicide_end("tests/GeneralStateTests/stCallCodes/callcallcode_01_SuicideEnd.json");
        fn callcallcall_000("tests/GeneralStateTests/stCallCodes/callcallcall_000.json");
        fn callcallcodecall_010_o_o_g_m_before("tests/GeneralStateTests/stCallCodes/callcallcodecall_010_OOGMBefore.json");
        fn callcodecallcodecall_110_suicide_middle("tests/GeneralStateTests/stCallCodes/callcodecallcodecall_110_SuicideMiddle.json");
        fn callcodecall_10_o_o_g_e("tests/GeneralStateTests/stCallCodes/callcodecall_10_OOGE.json");
        fn callcodecallcall_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallCodes/callcodecallcall_ABCB_RECURSIVE.json");
        fn callcode_in_initcode_to_empty_contract("tests/GeneralStateTests/stCallCodes/callcodeInInitcodeToEmptyContract.json");
        fn call_o_o_g_additional_gas_costs2("tests/GeneralStateTests/stCallCodes/call_OOG_additionalGasCosts2.json");
        fn callcallcodecallcode_011("tests/GeneralStateTests/stCallCodes/callcallcodecallcode_011.json");
        fn callcall_00_o_o_g_e("tests/GeneralStateTests/stCallCodes/callcall_00_OOGE.json");
        fn callcallcodecallcode_011_o_o_g_m_before("tests/GeneralStateTests/stCallCodes/callcallcodecallcode_011_OOGMBefore.json");
        fn callcode_check_p_c("tests/GeneralStateTests/stCallCodes/callcode_checkPC.json");
        fn touch_and_go("tests/GeneralStateTests/stCallCodes/touchAndGo.json");
        fn callcallcodecall_010_o_o_g_m_after("tests/GeneralStateTests/stCallCodes/callcallcodecall_010_OOGMAfter.json");
        fn callcodecallcodecallcode_111_o_o_g_e("tests/GeneralStateTests/stCallCodes/callcodecallcodecallcode_111_OOGE.json");
        fn callcodecallcall_100("tests/GeneralStateTests/stCallCodes/callcodecallcall_100.json");
        fn callcodecallcallcode_101_o_o_g_m_before("tests/GeneralStateTests/stCallCodes/callcodecallcallcode_101_OOGMBefore.json");
        fn callcodecallcallcode_101_suicide_middle("tests/GeneralStateTests/stCallCodes/callcodecallcallcode_101_SuicideMiddle.json");
        fn callcodecall_10_suicide_end("tests/GeneralStateTests/stCallCodes/callcodecall_10_SuicideEnd.json");
        fn callcallcall_000_o_o_g_e("tests/GeneralStateTests/stCallCodes/callcallcall_000_OOGE.json");
        fn callcallcodecall_010_suicide_end("tests/GeneralStateTests/stCallCodes/callcallcodecall_010_SuicideEnd.json");
        fn callcode_emptycontract("tests/GeneralStateTests/stCallCodes/callcodeEmptycontract.json");
        fn callcodecallcodecall_110_o_o_g_m_after("tests/GeneralStateTests/stCallCodes/callcodecallcodecall_110_OOGMAfter.json");
        fn callcallcodecallcode_011_o_o_g_m_after("tests/GeneralStateTests/stCallCodes/callcallcodecallcode_011_OOGMAfter.json");
        fn callcallcodecall_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallCodes/callcallcodecall_ABCB_RECURSIVE.json");
        fn callcallcodecallcode_011_o_o_g_e("tests/GeneralStateTests/stCallCodes/callcallcodecallcode_011_OOGE.json");
        fn callcodecallcall_100_o_o_g_m_after("tests/GeneralStateTests/stCallCodes/callcodecallcall_100_OOGMAfter.json");
        fn callcall_00_suicide_end("tests/GeneralStateTests/stCallCodes/callcall_00_SuicideEnd.json");
        fn callcode_in_initcode_to_existing_contract("tests/GeneralStateTests/stCallCodes/callcodeInInitcodeToExistingContract.json");
        fn callcallcodecallcode_011_suicide_end("tests/GeneralStateTests/stCallCodes/callcallcodecallcode_011_SuicideEnd.json");
        fn call_o_o_g_additional_gas_costs1("tests/GeneralStateTests/stCallCodes/call_OOG_additionalGasCosts1.json");
        fn callcodecallcodecallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallCodes/callcodecallcodecallcode_ABCB_RECURSIVE.json");
        fn callcodecallcallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallCodes/callcodecallcallcode_ABCB_RECURSIVE.json");
        fn callcodecallcallcode_101_suicide_end("tests/GeneralStateTests/stCallCodes/callcodecallcallcode_101_SuicideEnd.json");
        fn callcodecallcodecallcode_111("tests/GeneralStateTests/stCallCodes/callcodecallcodecallcode_111.json");
        fn callcallcall_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallCodes/callcallcall_ABCB_RECURSIVE.json");
        fn callcallcodecall_010_o_o_g_e("tests/GeneralStateTests/stCallCodes/callcallcodecall_010_OOGE.json");
        fn callcallcallcode_001_o_o_g_e("tests/GeneralStateTests/stCallCodes/callcallcallcode_001_OOGE.json");
        fn callcall_00("tests/GeneralStateTests/stCallCodes/callcall_00.json");
        fn callcall_00_o_o_g_e_value_transfer("tests/GeneralStateTests/stCallCodes/callcall_00_OOGE_valueTransfer.json");
        fn callcodecallcallcode_101_o_o_g_m_after("tests/GeneralStateTests/stCallCodes/callcodecallcallcode_101_OOGMAfter.json");
        fn callcodecallcall_100_suicide_middle("tests/GeneralStateTests/stCallCodes/callcodecallcall_100_SuicideMiddle.json");
        fn callcodecallcodecall_110("tests/GeneralStateTests/stCallCodes/callcodecallcodecall_110.json");
        fn callcallcodecallcode_011_suicide_middle("tests/GeneralStateTests/stCallCodes/callcallcodecallcode_011_SuicideMiddle.json");
        fn callcodecallcode_11_suicide_end("tests/GeneralStateTests/stCallCodes/callcodecallcode_11_SuicideEnd.json");
        fn callcallcallcode_001_suicide_middle("tests/GeneralStateTests/stCallCodes/callcallcallcode_001_SuicideMiddle.json");
        fn callcallcallcode_001_o_o_g_m_before("tests/GeneralStateTests/stCallCodes/callcallcallcode_001_OOGMBefore.json");
        fn callcodecallcodecallcode_111_suicide_middle("tests/GeneralStateTests/stCallCodes/callcodecallcodecallcode_111_SuicideMiddle.json");
        fn callcodecallcall_100_o_o_g_m_before("tests/GeneralStateTests/stCallCodes/callcodecallcall_100_OOGMBefore.json");
        fn callcallcodecall_010_suicide_middle("tests/GeneralStateTests/stCallCodes/callcallcodecall_010_SuicideMiddle.json");
        fn callcodecallcode_11_o_o_g_e("tests/GeneralStateTests/stCallCodes/callcodecallcode_11_OOGE.json");
        fn callcodecallcodecallcode_111_suicide_end("tests/GeneralStateTests/stCallCodes/callcodecallcodecallcode_111_SuicideEnd.json");
        fn callcallcode_01_o_o_g_e("tests/GeneralStateTests/stCallCodes/callcallcode_01_OOGE.json");
        fn callcodecallcodecall_110_o_o_g_m_before("tests/GeneralStateTests/stCallCodes/callcodecallcodecall_110_OOGMBefore.json");
        fn callcallcall_000_suicide_end("tests/GeneralStateTests/stCallCodes/callcallcall_000_SuicideEnd.json");
        fn callcallcodecallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallCodes/callcallcodecallcode_ABCB_RECURSIVE.json");
        fn callcode_in_initcode_to_existing_contract_with_value_transfer("tests/GeneralStateTests/stCallCodes/callcodeInInitcodeToExistingContractWithValueTransfer.json");
        fn callcallcodecall_010("tests/GeneralStateTests/stCallCodes/callcallcodecall_010.json");
        fn callcode_in_initcode_to_exis_contract_with_v_transfer_n_e_money("tests/GeneralStateTests/stCallCodes/callcodeInInitcodeToExisContractWithVTransferNEMoney.json");
        fn callcodecall_10("tests/GeneralStateTests/stCallCodes/callcodecall_10.json");
        fn callcodecallcodecall_110_o_o_g_e("tests/GeneralStateTests/stCallCodes/callcodecallcodecall_110_OOGE.json");
        fn callcodecallcallcode_101_o_o_g_e("tests/GeneralStateTests/stCallCodes/callcodecallcallcode_101_OOGE.json");
        fn callcode_dynamic_code2_self_call("tests/GeneralStateTests/stCallCodes/callcodeDynamicCode2SelfCall.json");
    }
}

mod st_pre_compiled_contracts2 {
    define_tests! {

        // --- ALL PASS ---
        fn c_a_l_l_c_o_d_e_identity_2("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEIdentity_2.json");
        fn c_a_l_l_c_o_d_e_ecrecover_h_prefixed0("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecoverH_prefixed0.json");
        fn c_a_l_l_c_o_d_e_ecrecover0_complete_return_value("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecover0_completeReturnValue.json");
        fn call_sha256_5("tests/GeneralStateTests/stPreCompiledContracts2/CallSha256_5.json");
        fn call_identity_4("tests/GeneralStateTests/stPreCompiledContracts2/CallIdentity_4.json");
        fn c_a_l_l_c_o_d_e_ripemd160_0("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODERipemd160_0.json");
        fn c_a_l_l_c_o_d_e_sha256_1("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODESha256_1.json");
        fn call_ecrecover0("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecover0.json");
        fn c_a_l_l_c_o_d_e_ecrecover0_0input("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecover0_0input.json");
        fn c_a_l_l_c_o_d_e_ripemd160_3_prefixed0("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODERipemd160_3_prefixed0.json");
        fn c_a_l_l_c_o_d_e_identity_4_gas17("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEIdentity_4_gas17.json");
        fn call_sha256_1_nonzero_value("tests/GeneralStateTests/stPreCompiledContracts2/CallSha256_1_nonzeroValue.json");
        fn call_ecrecover1("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecover1.json");
        fn c_a_l_l_c_o_d_e_sha256_0("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODESha256_0.json");
        fn c_a_l_l_c_o_d_e_ripemd160_1("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODERipemd160_1.json");
        fn call_identity_5("tests/GeneralStateTests/stPreCompiledContracts2/CallIdentity_5.json");
        fn call_ecrecover_invalid_signature("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecoverInvalidSignature.json");
        fn call_sha256_4("tests/GeneralStateTests/stPreCompiledContracts2/CallSha256_4.json");
        fn call_ecrecover0_gas3000("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecover0_gas3000.json");
        fn call_ecrecover0_no_gas("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecover0_NoGas.json");
        fn call_identity_1_nonzero_value("tests/GeneralStateTests/stPreCompiledContracts2/CallIdentity_1_nonzeroValue.json");
        fn c_a_l_l_c_o_d_e_identity_3("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEIdentity_3.json");
        fn call_identity_2("tests/GeneralStateTests/stPreCompiledContracts2/CallIdentity_2.json");
        fn call_ecrecover0_overlapping_input_output("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecover0_overlappingInputOutput.json");
        fn modexp_0_0_0_22000("tests/GeneralStateTests/stPreCompiledContracts2/modexp_0_0_0_22000.json");
        fn call_identity_4_gas18("tests/GeneralStateTests/stPreCompiledContracts2/CallIdentity_4_gas18.json");
        fn c_a_l_l_c_o_d_e_identity_4("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEIdentity_4.json");
        fn c_a_l_l_c_o_d_e_sha256_3_postfix0("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODESha256_3_postfix0.json");
        fn c_a_l_l_c_o_d_e_sha256_3_prefix0("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODESha256_3_prefix0.json");
        fn call_ripemd160_0("tests/GeneralStateTests/stPreCompiledContracts2/CallRipemd160_0.json");
        fn call_sha256_3("tests/GeneralStateTests/stPreCompiledContracts2/CallSha256_3.json");
        fn c_a_l_l_c_o_d_e_ripemd160_4_gas719("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODERipemd160_4_gas719.json");
        fn c_a_l_l_c_o_d_e_ecrecover0("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecover0.json");
        fn call_ecrecover_unrecoverable_key("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecoverUnrecoverableKey.json");
        fn c_a_l_l_c_o_d_e_ecrecover1("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecover1.json");
        fn call_ripemd160_1("tests/GeneralStateTests/stPreCompiledContracts2/CallRipemd160_1.json");
        fn call_sha256_2("tests/GeneralStateTests/stPreCompiledContracts2/CallSha256_2.json");
        fn call_ecrecover_h_prefixed0("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecoverH_prefixed0.json");
        fn c_a_l_l_c_o_d_e_identity_5("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEIdentity_5.json");
        fn c_a_l_l_c_o_d_e_ecrecover0_gas2999("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecover0_Gas2999.json");
        fn c_a_l_l_c_o_d_e_ecrecover_v_prefixedf0("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecoverV_prefixedf0.json");
        fn c_a_l_l_c_o_d_e_ripemd160_3_postfixed0("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODERipemd160_3_postfixed0.json");
        fn call_ecrecover_check_length("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecoverCheckLength.json");
        fn call_identity_6_input_shorter_than_output("tests/GeneralStateTests/stPreCompiledContracts2/CallIdentity_6_inputShorterThanOutput.json");
        fn call_identity_3("tests/GeneralStateTests/stPreCompiledContracts2/CallIdentity_3.json");
        fn call_sha256_4_gas99("tests/GeneralStateTests/stPreCompiledContracts2/CallSha256_4_gas99.json");
        fn c_a_l_l_c_o_d_e_ecrecover_v_prefixed0("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecoverV_prefixed0.json");
        fn c_a_l_l_c_o_d_e_blake2f("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEBlake2f.json");
        fn call_ecrecover_s_prefixed0("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecoverS_prefixed0.json");
        fn c_a_l_l_c_o_d_e_ecrecover0_no_gas("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecover0_NoGas.json");
        fn c_a_l_l_c_o_d_e_sha256_5("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODESha256_5.json");
        fn c_a_l_l_c_o_d_e_sha256_4_gas99("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODESha256_4_gas99.json");
        fn c_a_l_l_c_o_d_e_ripemd160_4("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODERipemd160_4.json");
        fn call_ecrecover80("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecover80.json");
        fn c_a_l_l_c_o_d_e_ecrecover2("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecover2.json");
        fn call_sha256_1("tests/GeneralStateTests/stPreCompiledContracts2/CallSha256_1.json");
        fn call_ripemd160_2("tests/GeneralStateTests/stPreCompiledContracts2/CallRipemd160_2.json");
        fn c_a_l_l_c_o_d_e_ecrecover0_gas3000("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecover0_gas3000.json");
        fn c_a_l_l_c_o_d_e_identitiy_0("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEIdentitiy_0.json");
        fn call_ecrecover_overflow("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecover_Overflow.json");
        fn call_sha256_3_postfix0("tests/GeneralStateTests/stPreCompiledContracts2/CallSha256_3_postfix0.json");
        fn call_ecrecover_check_length_wrong_v("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecoverCheckLengthWrongV.json");
        fn c_a_l_l_c_o_d_e_identitiy_1("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEIdentitiy_1.json");
        fn call_identity_4_gas17("tests/GeneralStateTests/stPreCompiledContracts2/CallIdentity_4_gas17.json");
        fn call_sha256_0("tests/GeneralStateTests/stPreCompiledContracts2/CallSha256_0.json");
        fn call_ripemd160_3("tests/GeneralStateTests/stPreCompiledContracts2/CallRipemd160_3.json");
        fn call_ripemd160_3_postfixed0("tests/GeneralStateTests/stPreCompiledContracts2/CallRipemd160_3_postfixed0.json");
        fn c_a_l_l_c_o_d_e_ecrecover3("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecover3.json");
        fn c_a_l_l_c_o_d_e_ripemd160_5("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODERipemd160_5.json");
        fn c_a_l_l_c_o_d_e_sha256_4("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODESha256_4.json");
        fn modexp_random_input("tests/GeneralStateTests/stPreCompiledContracts2/modexpRandomInput.json");
        fn call_ecrecover_r_prefixed0("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecoverR_prefixed0.json");
        fn call_ripemd160_4("tests/GeneralStateTests/stPreCompiledContracts2/CallRipemd160_4.json");
        fn c_a_l_l_c_o_d_e_ecrecover80("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecover80.json");
        fn ecrecover_weird_v("tests/GeneralStateTests/stPreCompiledContracts2/ecrecoverWeirdV.json");
        fn c_a_l_l_c_o_d_e_identity_1_nonzero_value("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEIdentity_1_nonzeroValue.json");
        fn call_ecrecover0_gas2999("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecover0_Gas2999.json");
        fn call_identitiy_0("tests/GeneralStateTests/stPreCompiledContracts2/CallIdentitiy_0.json");
        fn call_ecrecover2("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecover2.json");
        fn c_a_l_l_c_o_d_e_sha256_1_nonzero_value("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODESha256_1_nonzeroValue.json");
        fn modexp_0_0_0_35000("tests/GeneralStateTests/stPreCompiledContracts2/modexp_0_0_0_35000.json");
        fn modexp_0_0_0_25000("tests/GeneralStateTests/stPreCompiledContracts2/modexp_0_0_0_25000.json");
        fn call_ecrecover0_0input("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecover0_0input.json");
        fn c_a_l_l_c_o_d_e_sha256_3("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODESha256_3.json");
        fn c_a_l_l_c_o_d_e_ripemd160_2("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODERipemd160_2.json");
        fn modexp_0_0_0_20500("tests/GeneralStateTests/stPreCompiledContracts2/modexp_0_0_0_20500.json");
        fn c_a_l_l_c_o_d_e_ecrecover_r_prefixed0("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecoverR_prefixed0.json");
        fn call_ecrecover_v_prefixed0("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecoverV_prefixed0.json");
        fn c_a_l_l_c_o_d_e_ripemd160_3("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODERipemd160_3.json");
        fn c_a_l_l_c_o_d_e_ecrecover_s_prefixed0("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecoverS_prefixed0.json");
        fn c_a_l_l_c_o_d_e_sha256_2("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODESha256_2.json");
        fn call_sha256_3_prefix0("tests/GeneralStateTests/stPreCompiledContracts2/CallSha256_3_prefix0.json");
        fn c_a_l_l_blake2f("tests/GeneralStateTests/stPreCompiledContracts2/CALLBlake2f.json");
        fn ecrecover_short_buff("tests/GeneralStateTests/stPreCompiledContracts2/ecrecoverShortBuff.json");
        fn call_ecrecover3("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecover3.json");
        fn call_ripemd160_3_prefixed0("tests/GeneralStateTests/stPreCompiledContracts2/CallRipemd160_3_prefixed0.json");
        fn call_identitiy_1("tests/GeneralStateTests/stPreCompiledContracts2/CallIdentitiy_1.json");
        fn call_ripemd160_4_gas719("tests/GeneralStateTests/stPreCompiledContracts2/CallRipemd160_4_gas719.json");
        fn c_a_l_l_c_o_d_e_ecrecover0_overlapping_input_output("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEEcrecover0_overlappingInputOutput.json");
        fn call_ripemd160_5("tests/GeneralStateTests/stPreCompiledContracts2/CallRipemd160_5.json");
        fn c_a_l_l_c_o_d_e_identity_4_gas18("tests/GeneralStateTests/stPreCompiledContracts2/CALLCODEIdentity_4_gas18.json");
        fn call_ecrecover0_complete_return_value("tests/GeneralStateTests/stPreCompiledContracts2/CallEcrecover0_completeReturnValue.json");
    }
}

mod st_zero_calls_test {
    define_tests! {

        // --- ALL PASS ---
        fn zero_value_c_a_l_l_c_o_d_e("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_CALLCODE.json");
        fn zero_value_transaction_c_a_l_lwith_data_to_empty_paris("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_TransactionCALLwithData_ToEmpty_Paris.json");
        fn zero_value_c_a_l_l_to_non_zero_balance("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_CALL_ToNonZeroBalance.json");
        fn zero_value_d_e_l_e_g_a_t_e_c_a_l_l("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_DELEGATECALL.json");
        fn zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_non_zero_balance("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_DELEGATECALL_ToNonZeroBalance.json");
        fn zero_value_c_a_l_l("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_CALL.json");
        fn zero_value_transaction_c_a_l_lwith_data_to_non_zero_balance("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_TransactionCALLwithData_ToNonZeroBalance.json");
        fn zero_value_c_a_l_l_c_o_d_e_to_one_storage_key("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_CALLCODE_ToOneStorageKey.json");
        fn zero_value_transaction_c_a_l_l_to_empty_paris("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_TransactionCALL_ToEmpty_Paris.json");
        fn zero_value_transaction_c_a_l_l_to_one_storage_key("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_TransactionCALL_ToOneStorageKey.json");
        fn zero_value_transaction_c_a_l_lwith_data_to_empty("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_TransactionCALLwithData_ToEmpty.json");
        fn zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_one_storage_key("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_DELEGATECALL_ToOneStorageKey.json");
        fn zero_value_transaction_c_a_l_l_to_empty("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_TransactionCALL_ToEmpty.json");
        fn zero_value_s_ui_c_id_e_to_one_storage_key_paris("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_SUICIDE_ToOneStorageKey_Paris.json");
        fn zero_value_transaction_c_a_l_l_to_one_storage_key_paris("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_TransactionCALL_ToOneStorageKey_Paris.json");
        fn zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_empty("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_DELEGATECALL_ToEmpty.json");
        fn zero_value_c_a_l_l_to_one_storage_key("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_CALL_ToOneStorageKey.json");
        fn zero_value_c_a_l_l_c_o_d_e_to_empty("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_CALLCODE_ToEmpty.json");
        fn zero_value_s_ui_c_id_e_to_empty_paris("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_SUICIDE_ToEmpty_Paris.json");
        fn zero_value_c_a_l_l_to_empty("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_CALL_ToEmpty.json");
        fn zero_value_transaction_c_a_l_l("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_TransactionCALL.json");
        fn zero_value_transaction_c_a_l_lwith_data_to_one_storage_key_paris("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_TransactionCALLwithData_ToOneStorageKey_Paris.json");
        fn zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_empty_paris("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_DELEGATECALL_ToEmpty_Paris.json");
        fn zero_value_c_a_l_l_to_one_storage_key_paris("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_CALL_ToOneStorageKey_Paris.json");
        fn zero_value_s_ui_c_id_e_to_empty("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_SUICIDE_ToEmpty.json");
        fn zero_value_c_a_l_l_c_o_d_e_to_non_zero_balance("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_CALLCODE_ToNonZeroBalance.json");
        fn zero_value_transaction_c_a_l_lwith_data("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_TransactionCALLwithData.json");
        fn zero_value_s_ui_c_id_e_to_non_zero_balance("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_SUICIDE_ToNonZeroBalance.json");
        fn zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_one_storage_key_paris("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_DELEGATECALL_ToOneStorageKey_Paris.json");
        fn zero_value_c_a_l_l_c_o_d_e_to_empty_paris("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_CALLCODE_ToEmpty_Paris.json");
        fn zero_value_c_a_l_l_c_o_d_e_to_one_storage_key_paris("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_CALLCODE_ToOneStorageKey_Paris.json");
        fn zero_value_s_ui_c_id_e_to_one_storage_key("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_SUICIDE_ToOneStorageKey.json");
        fn zero_value_transaction_c_a_l_lwith_data_to_one_storage_key("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_TransactionCALLwithData_ToOneStorageKey.json");
        fn zero_value_c_a_l_l_to_empty_paris("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_CALL_ToEmpty_Paris.json");
        fn zero_value_transaction_c_a_l_l_to_non_zero_balance("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_TransactionCALL_ToNonZeroBalance.json");
        fn zero_value_s_ui_c_id_e("tests/GeneralStateTests/stZeroCallsTest/ZeroValue_SUICIDE.json");
    }
}

mod st_bad_opcode {
    define_tests! {

        // --- ALL PASS ---
        fn opc_a_c_diff_places("tests/GeneralStateTests/stBadOpcode/opcACDiffPlaces.json");
        fn opc2_f_diff_places("tests/GeneralStateTests/stBadOpcode/opc2FDiffPlaces.json");
        fn opc_f_b_diff_places("tests/GeneralStateTests/stBadOpcode/opcFBDiffPlaces.json");
        fn opc4_b_diff_places("tests/GeneralStateTests/stBadOpcode/opc4BDiffPlaces.json");
        fn opc_c6_diff_places("tests/GeneralStateTests/stBadOpcode/opcC6DiffPlaces.json");
        fn opc_b3_diff_places("tests/GeneralStateTests/stBadOpcode/opcB3DiffPlaces.json");
        fn opc_e2_diff_places("tests/GeneralStateTests/stBadOpcode/opcE2DiffPlaces.json");
        fn opc_d7_diff_places("tests/GeneralStateTests/stBadOpcode/opcD7DiffPlaces.json");
        fn opc_d8_diff_places("tests/GeneralStateTests/stBadOpcode/opcD8DiffPlaces.json");
        fn opc2_a_diff_places("tests/GeneralStateTests/stBadOpcode/opc2ADiffPlaces.json");
        fn opc_a_d_diff_places("tests/GeneralStateTests/stBadOpcode/opcADDiffPlaces.json");
        fn opc_c9_diff_places("tests/GeneralStateTests/stBadOpcode/opcC9DiffPlaces.json");
        fn undefined_opcode_first_byte("tests/GeneralStateTests/stBadOpcode/undefinedOpcodeFirstByte.json");
        fn opc4_e_diff_places("tests/GeneralStateTests/stBadOpcode/opc4EDiffPlaces.json");
        fn opc_f_e_diff_places("tests/GeneralStateTests/stBadOpcode/opcFEDiffPlaces.json");
        fn opc_b4_diff_places("tests/GeneralStateTests/stBadOpcode/opcB4DiffPlaces.json");
        fn opc_c1_diff_places("tests/GeneralStateTests/stBadOpcode/opcC1DiffPlaces.json");
        fn opc_d0_diff_places("tests/GeneralStateTests/stBadOpcode/opcD0DiffPlaces.json");
        fn opc_e5_diff_places("tests/GeneralStateTests/stBadOpcode/opcE5DiffPlaces.json");
        fn opc_c0_diff_places("tests/GeneralStateTests/stBadOpcode/opcC0DiffPlaces.json");
        fn opc_b5_diff_places("tests/GeneralStateTests/stBadOpcode/opcB5DiffPlaces.json");
        fn eip2315_not_removed("tests/GeneralStateTests/stBadOpcode/eip2315NotRemoved.json");
        fn opc_e4_diff_places("tests/GeneralStateTests/stBadOpcode/opcE4DiffPlaces.json");
        fn opc_d1_diff_places("tests/GeneralStateTests/stBadOpcode/opcD1DiffPlaces.json");
        fn opc_a_e_diff_places("tests/GeneralStateTests/stBadOpcode/opcAEDiffPlaces.json");
        fn opc_d9_diff_places("tests/GeneralStateTests/stBadOpcode/opcD9DiffPlaces.json");
        fn opc4_d_diff_places("tests/GeneralStateTests/stBadOpcode/opc4DDiffPlaces.json");
        fn opc_c8_diff_places("tests/GeneralStateTests/stBadOpcode/opcC8DiffPlaces.json");
        fn opc_b2_diff_places("tests/GeneralStateTests/stBadOpcode/opcB2DiffPlaces.json");
        fn opc_c7_diff_places("tests/GeneralStateTests/stBadOpcode/opcC7DiffPlaces.json");
        fn opc_d6_diff_places("tests/GeneralStateTests/stBadOpcode/opcD6DiffPlaces.json");
        fn opc_e3_diff_places("tests/GeneralStateTests/stBadOpcode/opcE3DiffPlaces.json");
        fn opc_a_b_diff_places("tests/GeneralStateTests/stBadOpcode/opcABDiffPlaces.json");
        fn opc5_f_diff_places("tests/GeneralStateTests/stBadOpcode/opc5FDiffPlaces.json");
        fn opc4_c_diff_places("tests/GeneralStateTests/stBadOpcode/opc4CDiffPlaces.json");
        fn opc_f_c_diff_places("tests/GeneralStateTests/stBadOpcode/opcFCDiffPlaces.json");
        fn opc_d4_diff_places("tests/GeneralStateTests/stBadOpcode/opcD4DiffPlaces.json");
        fn opc_e1_diff_places("tests/GeneralStateTests/stBadOpcode/opcE1DiffPlaces.json");
        fn opc_b0_diff_places("tests/GeneralStateTests/stBadOpcode/opcB0DiffPlaces.json");
        fn opc_c5_diff_places("tests/GeneralStateTests/stBadOpcode/opcC5DiffPlaces.json");
        fn opc5_d_diff_places("tests/GeneralStateTests/stBadOpcode/opc5DDiffPlaces.json");
        fn opc4_a_diff_places("tests/GeneralStateTests/stBadOpcode/opc4ADiffPlaces.json");
        fn opc_b8_diff_places("tests/GeneralStateTests/stBadOpcode/opcB8DiffPlaces.json");
        fn opc2_e_diff_places("tests/GeneralStateTests/stBadOpcode/opc2EDiffPlaces.json");
        fn opc_e9_diff_places("tests/GeneralStateTests/stBadOpcode/opcE9DiffPlaces.json");
        fn opc_e6_diff_places("tests/GeneralStateTests/stBadOpcode/opcE6DiffPlaces.json");
        fn opc_d3_diff_places("tests/GeneralStateTests/stBadOpcode/opcD3DiffPlaces.json");
        fn opc_c2_diff_places("tests/GeneralStateTests/stBadOpcode/opcC2DiffPlaces.json");
        fn opc_b7_diff_places("tests/GeneralStateTests/stBadOpcode/opcB7DiffPlaces.json");
        fn opc4_f_diff_places("tests/GeneralStateTests/stBadOpcode/opc4FDiffPlaces.json");
        fn opc5_c_diff_places("tests/GeneralStateTests/stBadOpcode/opc5CDiffPlaces.json");
        fn opc2_b_diff_places("tests/GeneralStateTests/stBadOpcode/opc2BDiffPlaces.json");
        fn opc2_c_diff_places("tests/GeneralStateTests/stBadOpcode/opc2CDiffPlaces.json");
        fn opc_a_f_diff_places("tests/GeneralStateTests/stBadOpcode/opcAFDiffPlaces.json");
        fn opc_d2_diff_places("tests/GeneralStateTests/stBadOpcode/opcD2DiffPlaces.json");
        fn opc_e7_diff_places("tests/GeneralStateTests/stBadOpcode/opcE7DiffPlaces.json");
        fn opc_b6_diff_places("tests/GeneralStateTests/stBadOpcode/opcB6DiffPlaces.json");
        fn opc_c3_diff_places("tests/GeneralStateTests/stBadOpcode/opcC3DiffPlaces.json");
        fn measure_gas("tests/GeneralStateTests/stBadOpcode/measureGas.json");
        fn opc_b9_diff_places("tests/GeneralStateTests/stBadOpcode/opcB9DiffPlaces.json");
        fn opc5_e_diff_places("tests/GeneralStateTests/stBadOpcode/opc5EDiffPlaces.json");
        fn opc_e8_diff_places("tests/GeneralStateTests/stBadOpcode/opcE8DiffPlaces.json");
        fn opc_a_a_diff_places("tests/GeneralStateTests/stBadOpcode/opcAADiffPlaces.json");
        fn opc2_d_diff_places("tests/GeneralStateTests/stBadOpcode/opc2DDiffPlaces.json");
        fn opc_e0_diff_places("tests/GeneralStateTests/stBadOpcode/opcE0DiffPlaces.json");
        fn opc_d5_diff_places("tests/GeneralStateTests/stBadOpcode/opcD5DiffPlaces.json");
        fn opc_c4_diff_places("tests/GeneralStateTests/stBadOpcode/opcC4DiffPlaces.json");
        fn opc_b1_diff_places("tests/GeneralStateTests/stBadOpcode/opcB1DiffPlaces.json");
        fn opc_d_a_diff_places("tests/GeneralStateTests/stBadOpcode/opcDADiffPlaces.json");
        fn opc_e_d_diff_places("tests/GeneralStateTests/stBadOpcode/opcEDDiffPlaces.json");
        fn opc28_diff_places("tests/GeneralStateTests/stBadOpcode/opc28DiffPlaces.json");
        fn opc0_e_diff_places("tests/GeneralStateTests/stBadOpcode/opc0EDiffPlaces.json");
        fn opc_b_e_diff_places("tests/GeneralStateTests/stBadOpcode/opcBEDiffPlaces.json");
        fn opc_a5_diff_places("tests/GeneralStateTests/stBadOpcode/opcA5DiffPlaces.json");
        fn opc_e_c_diff_places("tests/GeneralStateTests/stBadOpcode/opcECDiffPlaces.json");
        fn opc_d_f_diff_places("tests/GeneralStateTests/stBadOpcode/opcDFDiffPlaces.json");
        fn opc_b_b_diff_places("tests/GeneralStateTests/stBadOpcode/opcBBDiffPlaces.json");
        fn bad_opcodes("tests/GeneralStateTests/stBadOpcode/badOpcodes.json");
        fn opc27_diff_places("tests/GeneralStateTests/stBadOpcode/opc27DiffPlaces.json");
        // fn operation_diff_gas("tests/GeneralStateTests/stBadOpcode/operationDiffGas.json");
        fn opc26_diff_places("tests/GeneralStateTests/stBadOpcode/opc26DiffPlaces.json");
        fn opc_e_b_diff_places("tests/GeneralStateTests/stBadOpcode/opcEBDiffPlaces.json");
        fn opc1_f_diff_places("tests/GeneralStateTests/stBadOpcode/opc1FDiffPlaces.json");
        fn opc0_c_diff_places("tests/GeneralStateTests/stBadOpcode/opc0CDiffPlaces.json");
        fn opc_b_c_diff_places("tests/GeneralStateTests/stBadOpcode/opcBCDiffPlaces.json");
        fn opc_c_f_diff_places("tests/GeneralStateTests/stBadOpcode/opcCFDiffPlaces.json");
        fn opc21_diff_places("tests/GeneralStateTests/stBadOpcode/opc21DiffPlaces.json");
        fn opc29_diff_places("tests/GeneralStateTests/stBadOpcode/opc29DiffPlaces.json");
        fn invalid_diff_places("tests/GeneralStateTests/stBadOpcode/invalidDiffPlaces.json");
        fn opc_e_e_diff_places("tests/GeneralStateTests/stBadOpcode/opcEEDiffPlaces.json");
        fn opc_c_a_diff_places("tests/GeneralStateTests/stBadOpcode/opcCADiffPlaces.json");
        fn opc_b_d_diff_places("tests/GeneralStateTests/stBadOpcode/opcBDDiffPlaces.json");
        fn opc0_d_diff_places("tests/GeneralStateTests/stBadOpcode/opc0DDiffPlaces.json");
        fn opc23_diff_places("tests/GeneralStateTests/stBadOpcode/opc23DiffPlaces.json");
        fn opc_a6_diff_places("tests/GeneralStateTests/stBadOpcode/opcA6DiffPlaces.json");
        fn opc_f7_diff_places("tests/GeneralStateTests/stBadOpcode/opcF7DiffPlaces.json");
        fn opc_c_c_diff_places("tests/GeneralStateTests/stBadOpcode/opcCCDiffPlaces.json");
        fn opc_b_f_diff_places("tests/GeneralStateTests/stBadOpcode/opcBFDiffPlaces.json");
        fn opc0_f_diff_places("tests/GeneralStateTests/stBadOpcode/opc0FDiffPlaces.json");
        fn opc_d_b_diff_places("tests/GeneralStateTests/stBadOpcode/opcDBDiffPlaces.json");
        fn invalid_addr("tests/GeneralStateTests/stBadOpcode/invalidAddr.json");
        fn opc24_diff_places("tests/GeneralStateTests/stBadOpcode/opc24DiffPlaces.json");
        fn opc_f8_diff_places("tests/GeneralStateTests/stBadOpcode/opcF8DiffPlaces.json");
        fn opc_b_a_diff_places("tests/GeneralStateTests/stBadOpcode/opcBADiffPlaces.json");
        fn opc_c_d_diff_places("tests/GeneralStateTests/stBadOpcode/opcCDDiffPlaces.json");
        fn opc_a9_diff_places("tests/GeneralStateTests/stBadOpcode/opcA9DiffPlaces.json");
        fn opc_d_e_diff_places("tests/GeneralStateTests/stBadOpcode/opcDEDiffPlaces.json");
        fn opc_c_e_diff_places("tests/GeneralStateTests/stBadOpcode/opcCEDiffPlaces.json");
        fn opc1_e_diff_places("tests/GeneralStateTests/stBadOpcode/opc1EDiffPlaces.json");
        fn opc_f9_diff_places("tests/GeneralStateTests/stBadOpcode/opcF9DiffPlaces.json");
        fn opc49_diff_places("tests/GeneralStateTests/stBadOpcode/opc49DiffPlaces.json");
        fn opc_e_a_diff_places("tests/GeneralStateTests/stBadOpcode/opcEADiffPlaces.json");
        fn opc_d_d_diff_places("tests/GeneralStateTests/stBadOpcode/opcDDDiffPlaces.json");
        fn opc_a8_diff_places("tests/GeneralStateTests/stBadOpcode/opcA8DiffPlaces.json");
        fn opc25_diff_places("tests/GeneralStateTests/stBadOpcode/opc25DiffPlaces.json");
        fn opc_c_b_diff_places("tests/GeneralStateTests/stBadOpcode/opcCBDiffPlaces.json");
        fn opc_d_c_diff_places("tests/GeneralStateTests/stBadOpcode/opcDCDiffPlaces.json");
        fn opc_e_f_diff_places("tests/GeneralStateTests/stBadOpcode/opcEFDiffPlaces.json");
        fn opc_a7_diff_places("tests/GeneralStateTests/stBadOpcode/opcA7DiffPlaces.json");
        fn opc22_diff_places("tests/GeneralStateTests/stBadOpcode/opc22DiffPlaces.json");
        fn opc_f6_diff_places("tests/GeneralStateTests/stBadOpcode/opcF6DiffPlaces.json");
    }
}

mod st_memory_stress_test {
    define_tests! {

        // --- ALL PASS ---
        fn m_s_t_o_r_e_bounds2a("tests/GeneralStateTests/stMemoryStressTest/MSTORE_Bounds2a.json");
        fn c_a_l_l_c_o_d_e_bounds("tests/GeneralStateTests/stMemoryStressTest/CALLCODE_Bounds.json");
        fn r_e_t_u_r_n_bounds("tests/GeneralStateTests/stMemoryStressTest/RETURN_Bounds.json");
        fn m_l_o_a_d_bounds3("tests/GeneralStateTests/stMemoryStressTest/MLOAD_Bounds3.json");
        fn j_u_m_p_bounds2("tests/GeneralStateTests/stMemoryStressTest/JUMP_Bounds2.json");
        fn c_a_l_l_c_o_d_e_bounds2("tests/GeneralStateTests/stMemoryStressTest/CALLCODE_Bounds2.json");
        fn c_r_e_a_t_e_bounds("tests/GeneralStateTests/stMemoryStressTest/CREATE_Bounds.json");
        fn c_a_l_l_c_o_d_e_bounds3("tests/GeneralStateTests/stMemoryStressTest/CALLCODE_Bounds3.json");
        fn j_u_m_p_i_bounds("tests/GeneralStateTests/stMemoryStressTest/JUMPI_Bounds.json");
        fn m_l_o_a_d_bounds2("tests/GeneralStateTests/stMemoryStressTest/MLOAD_Bounds2.json");
        fn mload32bit_bound_return("tests/GeneralStateTests/stMemoryStressTest/mload32bitBound_return.json");
        fn m_s_t_o_r_e_bounds2("tests/GeneralStateTests/stMemoryStressTest/MSTORE_Bounds2.json");
        fn mload32bit_bound2("tests/GeneralStateTests/stMemoryStressTest/mload32bitBound2.json");
        fn c_a_l_l_c_o_d_e_bounds4("tests/GeneralStateTests/stMemoryStressTest/CALLCODE_Bounds4.json");
        fn static_c_a_l_l_bounds3("tests/GeneralStateTests/stMemoryStressTest/static_CALL_Bounds3.json");
        fn static_c_a_l_l_bounds2a("tests/GeneralStateTests/stMemoryStressTest/static_CALL_Bounds2a.json");
        fn c_a_l_l_bounds("tests/GeneralStateTests/stMemoryStressTest/CALL_Bounds.json");
        fn static_c_a_l_l_bounds2("tests/GeneralStateTests/stMemoryStressTest/static_CALL_Bounds2.json");
        fn m_l_o_a_d_bounds("tests/GeneralStateTests/stMemoryStressTest/MLOAD_Bounds.json");
        fn fill_stack("tests/GeneralStateTests/stMemoryStressTest/FillStack.json");
        fn d_e_l_e_g_a_t_e_c_a_l_l_bounds3("tests/GeneralStateTests/stMemoryStressTest/DELEGATECALL_Bounds3.json");
        fn mload32bit_bound("tests/GeneralStateTests/stMemoryStressTest/mload32bitBound.json");
        fn c_a_l_l_bounds2("tests/GeneralStateTests/stMemoryStressTest/CALL_Bounds2.json");
        fn d_e_l_e_g_a_t_e_c_a_l_l_bounds("tests/GeneralStateTests/stMemoryStressTest/DELEGATECALL_Bounds.json");
        fn c_a_l_l_bounds3("tests/GeneralStateTests/stMemoryStressTest/CALL_Bounds3.json");
        fn d_e_l_e_g_a_t_e_c_a_l_l_bounds2("tests/GeneralStateTests/stMemoryStressTest/DELEGATECALL_Bounds2.json");
        fn s_s_t_o_r_e_bounds("tests/GeneralStateTests/stMemoryStressTest/SSTORE_Bounds.json");
        fn c_r_e_a_t_e_bounds2("tests/GeneralStateTests/stMemoryStressTest/CREATE_Bounds2.json");
        fn mload32bit_bound_msize("tests/GeneralStateTests/stMemoryStressTest/mload32bitBound_Msize.json");
        fn static_c_a_l_l_bounds("tests/GeneralStateTests/stMemoryStressTest/static_CALL_Bounds.json");
        fn p_o_p_bounds("tests/GeneralStateTests/stMemoryStressTest/POP_Bounds.json");
        fn d_u_p_bounds("tests/GeneralStateTests/stMemoryStressTest/DUP_Bounds.json");
        fn s_l_o_a_d_bounds("tests/GeneralStateTests/stMemoryStressTest/SLOAD_Bounds.json");
        fn c_a_l_l_bounds2a("tests/GeneralStateTests/stMemoryStressTest/CALL_Bounds2a.json");
        fn m_s_t_o_r_e_bounds("tests/GeneralStateTests/stMemoryStressTest/MSTORE_Bounds.json");
        fn mload32bit_bound_return2("tests/GeneralStateTests/stMemoryStressTest/mload32bitBound_return2.json");
        fn j_u_m_p_bounds("tests/GeneralStateTests/stMemoryStressTest/JUMP_Bounds.json");
        fn c_r_e_a_t_e_bounds3("tests/GeneralStateTests/stMemoryStressTest/CREATE_Bounds3.json");
    }
}

mod st_shift {
    define_tests! {

        // --- ALL PASS ---
        fn shift_signed_combinations("tests/GeneralStateTests/stShift/shiftSignedCombinations.json");
        fn sar_2_254_254("tests/GeneralStateTests/stShift/sar_2^254_254.json");
        fn sar01("tests/GeneralStateTests/stShift/sar01.json");
        fn shl11("tests/GeneralStateTests/stShift/shl11.json");
        fn shr01("tests/GeneralStateTests/stShift/shr01.json");
        fn shr_1_256("tests/GeneralStateTests/stShift/shr_-1_256.json");
        fn shl10("tests/GeneralStateTests/stShift/shl10.json");
        fn sar00("tests/GeneralStateTests/stShift/sar00.json");
        fn sar_2_255_255("tests/GeneralStateTests/stShift/sar_2^255_255.json");
        fn shl01_ff("tests/GeneralStateTests/stShift/shl01-ff.json");
        fn sar_2_256_1_255("tests/GeneralStateTests/stShift/sar_2^256-1_255.json");
        fn shr11("tests/GeneralStateTests/stShift/shr11.json");
        fn shl01("tests/GeneralStateTests/stShift/shl01.json");
        fn sar_2_255_1_256("tests/GeneralStateTests/stShift/sar_2^255-1_256.json");
        fn sar11("tests/GeneralStateTests/stShift/sar11.json");
        fn shl_1_255("tests/GeneralStateTests/stShift/shl_-1_255.json");
        fn sar10("tests/GeneralStateTests/stShift/sar10.json");
        fn shr10("tests/GeneralStateTests/stShift/shr10.json");
        fn shr_2_255_255("tests/GeneralStateTests/stShift/shr_2^255_255.json");
        fn shr_1_0("tests/GeneralStateTests/stShift/shr_-1_0.json");
        fn shr_2_255_256("tests/GeneralStateTests/stShift/shr_2^255_256.json");
        fn shl_1_1("tests/GeneralStateTests/stShift/shl_-1_1.json");
        fn shl01_0101("tests/GeneralStateTests/stShift/shl01-0101.json");
        fn shift_combinations("tests/GeneralStateTests/stShift/shiftCombinations.json");
        fn sar_2_255_1_254("tests/GeneralStateTests/stShift/sar_2^255-1_254.json");
        fn sar_2_255_1_255("tests/GeneralStateTests/stShift/sar_2^255-1_255.json");
        fn shr_2_255_1("tests/GeneralStateTests/stShift/shr_2^255_1.json");
        fn shl_2_255_1_1("tests/GeneralStateTests/stShift/shl_2^255-1_1.json");
        fn shl_1_256("tests/GeneralStateTests/stShift/shl_-1_256.json");
        fn shl01_0100("tests/GeneralStateTests/stShift/shl01-0100.json");
        fn shl_1_0("tests/GeneralStateTests/stShift/shl_-1_0.json");
        fn sar_2_256_1_256("tests/GeneralStateTests/stShift/sar_2^256-1_256.json");
        fn shr_2_255_257("tests/GeneralStateTests/stShift/shr_2^255_257.json");
        fn shr_1_1("tests/GeneralStateTests/stShift/shr_-1_1.json");
        fn sar_2_255_256("tests/GeneralStateTests/stShift/sar_2^255_256.json");
        fn sar_2_255_1_248("tests/GeneralStateTests/stShift/sar_2^255-1_248.json");
        fn sar_2_256_1_1("tests/GeneralStateTests/stShift/sar_2^256-1_1.json");
        fn sar_2_256_1_0("tests/GeneralStateTests/stShift/sar_2^256-1_0.json");
        fn shr_1_255("tests/GeneralStateTests/stShift/shr_-1_255.json");
        fn sar_2_255_1("tests/GeneralStateTests/stShift/sar_2^255_1.json");
        fn sar_2_255_257("tests/GeneralStateTests/stShift/sar_2^255_257.json");
        fn sar_0_256_1("tests/GeneralStateTests/stShift/sar_0_256-1.json");
    }
}

mod st_special_test {
    define_tests! {

        // --- ALL PASS ---
        fn failed_create_reverts_deletion("tests/GeneralStateTests/stSpecialTest/FailedCreateRevertsDeletion.json");
        fn j_u_m_p_d_e_s_t_attackwith_jump("tests/GeneralStateTests/stSpecialTest/JUMPDEST_AttackwithJump.json");
        fn gas_price0("tests/GeneralStateTests/stSpecialTest/gasPrice0.json");
        fn eoa_empty_paris("tests/GeneralStateTests/stSpecialTest/eoaEmptyParis.json");
        fn tx_e1c174e2("tests/GeneralStateTests/stSpecialTest/tx_e1c174e2.json");
        fn selfdestruct_e_i_p2929("tests/GeneralStateTests/stSpecialTest/selfdestructEIP2929.json");
        fn deployment_error("tests/GeneralStateTests/stSpecialTest/deploymentError.json");
        fn failed_create_reverts_deletion_paris("tests/GeneralStateTests/stSpecialTest/FailedCreateRevertsDeletionParis.json");
        fn failed_tx_xcf416c53_paris("tests/GeneralStateTests/stSpecialTest/failed_tx_xcf416c53_Paris.json");
        fn failed_tx_xcf416c53("tests/GeneralStateTests/stSpecialTest/failed_tx_xcf416c53.json");
        fn stack_depth_limit_s_e_c("tests/GeneralStateTests/stSpecialTest/StackDepthLimitSEC.json");
        fn eoa_empty("tests/GeneralStateTests/stSpecialTest/eoaEmpty.json");
        fn j_u_m_p_d_e_s_t_attack("tests/GeneralStateTests/stSpecialTest/JUMPDEST_Attack.json");
        fn push32without_byte("tests/GeneralStateTests/stSpecialTest/push32withoutByte.json");
        fn overflow_gas_make_money("tests/GeneralStateTests/stSpecialTest/OverflowGasMakeMoney.json");
        fn block504980("tests/GeneralStateTests/stSpecialTest/block504980.json");
        fn make_money("tests/GeneralStateTests/stSpecialTest/makeMoney.json");
        fn sha3_deja("tests/GeneralStateTests/stSpecialTest/sha3_deja.json");
    }
}

mod st_call_create_call_code_test {
    define_tests! {

        // --- MOST PASS --- (4 fail because of some gas calc mismatch)
        fn call1024_balance_too_low("tests/GeneralStateTests/stCallCreateCallCodeTest/Call1024BalanceTooLow.json");
        fn callcode_output1("tests/GeneralStateTests/stCallCreateCallCodeTest/callcodeOutput1.json");
        fn callcode_lose_gas_o_o_g("tests/GeneralStateTests/stCallCreateCallCodeTest/CallcodeLoseGasOOG.json");
        fn create_init_fail_undefined_instruction2("tests/GeneralStateTests/stCallCreateCallCodeTest/createInitFailUndefinedInstruction2.json");
        fn create_j_s_example_contract("tests/GeneralStateTests/stCallCreateCallCodeTest/createJS_ExampleContract.json");
        fn create_init_fail_bad_jump_destination2("tests/GeneralStateTests/stCallCreateCallCodeTest/createInitFailBadJumpDestination2.json");
        fn create_name_registratorendowment_too_high("tests/GeneralStateTests/stCallCreateCallCodeTest/createNameRegistratorendowmentTooHigh.json");
        fn callcode1024_o_o_g("tests/GeneralStateTests/stCallCreateCallCodeTest/Callcode1024OOG.json");
        fn call_lose_gas_o_o_g("tests/GeneralStateTests/stCallCreateCallCodeTest/CallLoseGasOOG.json");
        fn call1024_o_o_g("tests/GeneralStateTests/stCallCreateCallCodeTest/Call1024OOG.json");
        fn create_name_registrator_per_txs("tests/GeneralStateTests/stCallCreateCallCodeTest/createNameRegistratorPerTxs.json");
        fn callcode_output3partial_fail("tests/GeneralStateTests/stCallCreateCallCodeTest/callcodeOutput3partialFail.json");
        fn call_with_high_value_and_gas_o_o_g("tests/GeneralStateTests/stCallCreateCallCodeTest/callWithHighValueAndGasOOG.json");
        fn call_with_high_value_o_o_gin_call("tests/GeneralStateTests/stCallCreateCallCodeTest/callWithHighValueOOGinCall.json");
        fn call_output1("tests/GeneralStateTests/stCallCreateCallCodeTest/callOutput1.json");
        fn create_init_fail_o_o_gduring_init("tests/GeneralStateTests/stCallCreateCallCodeTest/createInitFail_OOGduringInit.json");
        fn create_fail_balance_too_low("tests/GeneralStateTests/stCallCreateCallCodeTest/createFailBalanceTooLow.json");
        fn call1024_pre_calls("tests/GeneralStateTests/stCallCreateCallCodeTest/Call1024PreCalls.json");
        fn call_output3_fail("tests/GeneralStateTests/stCallCreateCallCodeTest/callOutput3Fail.json");
        fn call_output2("tests/GeneralStateTests/stCallCreateCallCodeTest/callOutput2.json");
        fn call_recursive_bomb_pre_call("tests/GeneralStateTests/stCallCreateCallCodeTest/CallRecursiveBombPreCall.json");
        fn call_output3("tests/GeneralStateTests/stCallCreateCallCodeTest/callOutput3.json");
        fn contract_creation_make_call_that_ask_more_gas_then_transaction_provided("tests/GeneralStateTests/stCallCreateCallCodeTest/contractCreationMakeCallThatAskMoreGasThenTransactionProvided.json");
        fn callcode_with_high_value_and_gas_o_o_g("tests/GeneralStateTests/stCallCreateCallCodeTest/callcodeWithHighValueAndGasOOG.json");
        fn create_init_fail_stack_underflow("tests/GeneralStateTests/stCallCreateCallCodeTest/createInitFailStackUnderflow.json");
        fn callcode_output3_fail("tests/GeneralStateTests/stCallCreateCallCodeTest/callcodeOutput3Fail.json");
        fn callcode_with_high_value("tests/GeneralStateTests/stCallCreateCallCodeTest/callcodeWithHighValue.json");
        fn create_j_s_no_collision("tests/GeneralStateTests/stCallCreateCallCodeTest/createJS_NoCollision.json");
        fn callcode1024_balance_too_low("tests/GeneralStateTests/stCallCreateCallCodeTest/Callcode1024BalanceTooLow.json");
        fn callcode_output3("tests/GeneralStateTests/stCallCreateCallCodeTest/callcodeOutput3.json");
        fn call_with_high_value_and_o_o_gat_tx_level("tests/GeneralStateTests/stCallCreateCallCodeTest/callWithHighValueAndOOGatTxLevel.json");
        fn call_with_high_value("tests/GeneralStateTests/stCallCreateCallCodeTest/callWithHighValue.json");
        fn callcode_output3partial("tests/GeneralStateTests/stCallCreateCallCodeTest/callcodeOutput3partial.json");
        fn create_init_fail_stack_size_larger_than1024("tests/GeneralStateTests/stCallCreateCallCodeTest/createInitFailStackSizeLargerThan1024.json");
        fn create_init_o_o_gfor_c_r_e_a_t_e("tests/GeneralStateTests/stCallCreateCallCodeTest/createInitOOGforCREATE.json");
        fn call_output3partial("tests/GeneralStateTests/stCallCreateCallCodeTest/callOutput3partial.json");
        fn create_name_registrator_per_txs_not_enough_gas("tests/GeneralStateTests/stCallCreateCallCodeTest/createNameRegistratorPerTxsNotEnoughGas.json");
        fn create_init_fail_bad_jump_destination("tests/GeneralStateTests/stCallCreateCallCodeTest/createInitFailBadJumpDestination.json");
        fn callcode_output2("tests/GeneralStateTests/stCallCreateCallCodeTest/callcodeOutput2.json");
        fn create_init_fail_undefined_instruction("tests/GeneralStateTests/stCallCreateCallCodeTest/createInitFailUndefinedInstruction.json");
        fn create_name_registrator_pre_store1_not_enough_gas("tests/GeneralStateTests/stCallCreateCallCodeTest/createNameRegistratorPreStore1NotEnoughGas.json");
        fn create_init_fail_o_o_gduring_init2("tests/GeneralStateTests/stCallCreateCallCodeTest/createInitFail_OOGduringInit2.json");
        fn call_output3partial_fail("tests/GeneralStateTests/stCallCreateCallCodeTest/callOutput3partialFail.json");
    }
}

mod st_quadratic_complexity_test {
    define_tests! {

        // --- ALL PASS --- (3 tests execute too long to wait)
        fn call20_kbytes_contract50_2("tests/GeneralStateTests/stQuadraticComplexityTest/Call20KbytesContract50_2.json");
        fn return50000_2("tests/GeneralStateTests/stQuadraticComplexityTest/Return50000_2.json");
        fn call20_kbytes_contract50_3("tests/GeneralStateTests/stQuadraticComplexityTest/Call20KbytesContract50_3.json");
        fn quadratic_complexity_solidity_call_data_copy("tests/GeneralStateTests/stQuadraticComplexityTest/QuadraticComplexitySolidity_CallDataCopy.json");
        fn call50000_ecrec("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000_ecrec.json");
        fn create1000("tests/GeneralStateTests/stQuadraticComplexityTest/Create1000.json");
        fn callcode50000("tests/GeneralStateTests/stQuadraticComplexityTest/Callcode50000.json");
        fn call1_m_b1024_calldepth("tests/GeneralStateTests/stQuadraticComplexityTest/Call1MB1024Calldepth.json");
        fn create1000_shnghai("tests/GeneralStateTests/stQuadraticComplexityTest/Create1000Shnghai.json");
        fn return50000("tests/GeneralStateTests/stQuadraticComplexityTest/Return50000.json");
        fn call50000_sha256("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000_sha256.json");
        fn call50000_identity2("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000_identity2.json");
        fn create1000_byzantium("tests/GeneralStateTests/stQuadraticComplexityTest/Create1000Byzantium.json");
        fn call50000_identity("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000_identity.json");
        fn call50000_rip160("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000_rip160.json");
        fn call20_kbytes_contract50_1("tests/GeneralStateTests/stQuadraticComplexityTest/Call20KbytesContract50_1.json");
        fn call50000("tests/GeneralStateTests/stQuadraticComplexityTest/Call50000.json");
    }
}

mod st_stack_tests {
    define_tests! {

        // --- ALL PASS --- (with disabled logging)
        fn stack_overflow("tests/GeneralStateTests/stStackTests/stackOverflow.json");
        fn stack_overflow_m1_p_u_s_h("tests/GeneralStateTests/stStackTests/stackOverflowM1PUSH.json");
        fn stack_overflow_m1("tests/GeneralStateTests/stStackTests/stackOverflowM1.json");
        fn stack_overflow_s_w_a_p("tests/GeneralStateTests/stStackTests/stackOverflowSWAP.json");
        fn stacksanity_s_w_a_p("tests/GeneralStateTests/stStackTests/stacksanitySWAP.json");
        fn stack_overflow_d_u_p("tests/GeneralStateTests/stStackTests/stackOverflowDUP.json");
        fn stack_overflow_m1_d_u_p("tests/GeneralStateTests/stStackTests/stackOverflowM1DUP.json");
        fn stack_overflow_p_u_s_h("tests/GeneralStateTests/stStackTests/stackOverflowPUSH.json");
        fn shallow_stack("tests/GeneralStateTests/stStackTests/shallowStack.json");
        fn underflow_test("tests/GeneralStateTests/stStackTests/underflowTest.json");
    }
}

mod st_solidity_test {
    define_tests! {

        // --- ALL PASS ---
        fn test_contract_suicide("tests/GeneralStateTests/stSolidityTest/TestContractSuicide.json");
        fn test_keywords("tests/GeneralStateTests/stSolidityTest/TestKeywords.json");
        fn test_cryptographic_functions("tests/GeneralStateTests/stSolidityTest/TestCryptographicFunctions.json");
        fn call_infinite_loop("tests/GeneralStateTests/stSolidityTest/CallInfiniteLoop.json");
        fn ambiguous_method("tests/GeneralStateTests/stSolidityTest/AmbiguousMethod.json");
        fn self_destruct("tests/GeneralStateTests/stSolidityTest/SelfDestruct.json");
        fn recursive_create_contracts("tests/GeneralStateTests/stSolidityTest/RecursiveCreateContracts.json");
        fn by_zero("tests/GeneralStateTests/stSolidityTest/ByZero.json");
        fn contract_inheritance("tests/GeneralStateTests/stSolidityTest/ContractInheritance.json");
        fn test_contract_interaction("tests/GeneralStateTests/stSolidityTest/TestContractInteraction.json");
        fn call_low_level_creates_solidity("tests/GeneralStateTests/stSolidityTest/CallLowLevelCreatesSolidity.json");
        fn test_overflow("tests/GeneralStateTests/stSolidityTest/TestOverflow.json");
        fn recursive_create_contracts_create4_contracts("tests/GeneralStateTests/stSolidityTest/RecursiveCreateContractsCreate4Contracts.json");
        fn test_structures_and_variabless("tests/GeneralStateTests/stSolidityTest/TestStructuresAndVariabless.json");
        fn test_store_gas_prices("tests/GeneralStateTests/stSolidityTest/TestStoreGasPrices.json");
        fn test_block_and_transaction_properties("tests/GeneralStateTests/stSolidityTest/TestBlockAndTransactionProperties.json");
        fn create_contract_from_method("tests/GeneralStateTests/stSolidityTest/CreateContractFromMethod.json");
        fn call_recursive_methods("tests/GeneralStateTests/stSolidityTest/CallRecursiveMethods.json");
    }
}

#[allow(non_snake_case)]
mod st_memory_test {
    define_tests! {

        // --- ALL PASS ---
        fn mem64kb_single_byte_32("tests/GeneralStateTests/stMemoryTest/mem64kb_singleByte-32.json");
        fn mload8bit_bound("tests/GeneralStateTests/stMemoryTest/mload8bitBound.json");
        fn stack_limit_push32_1025("tests/GeneralStateTests/stMemoryTest/stackLimitPush32_1025.json");
        fn log2_dejavu("tests/GeneralStateTests/stMemoryTest/log2_dejavu.json");
        fn codecopy_dejavu("tests/GeneralStateTests/stMemoryTest/codecopy_dejavu.json");
        fn mem64kb_32("tests/GeneralStateTests/stMemoryTest/mem64kb-32.json");
        fn mem32kb_single_byte_33("tests/GeneralStateTests/stMemoryTest/mem32kb_singleByte-33.json");
        fn mstroe8_dejavu("tests/GeneralStateTests/stMemoryTest/mstroe8_dejavu.json");
        fn codecopy_dejavu2("tests/GeneralStateTests/stMemoryTest/codecopy_dejavu2.json");
        fn sha3_dejavu("tests/GeneralStateTests/stMemoryTest/sha3_dejavu.json");
        fn mem32kb_32("tests/GeneralStateTests/stMemoryTest/mem32kb-32.json");
        fn mem64kb__1("tests/GeneralStateTests/stMemoryTest/mem64kb+1.json");
        fn mem32kb("tests/GeneralStateTests/stMemoryTest/mem32kb.json");
        fn mem32kb_33("tests/GeneralStateTests/stMemoryTest/mem32kb-33.json");
        fn buffer("tests/GeneralStateTests/stMemoryTest/buffer.json");
        fn mem64kb_33("tests/GeneralStateTests/stMemoryTest/mem64kb-33.json");
        fn mem32kb_single_byte_32("tests/GeneralStateTests/stMemoryTest/mem32kb_singleByte-32.json");
        fn stack_limit_push32_1024("tests/GeneralStateTests/stMemoryTest/stackLimitPush32_1024.json");
        fn mem64kb_1("tests/GeneralStateTests/stMemoryTest/mem64kb-1.json");
        fn stack_limit_gas_1023("tests/GeneralStateTests/stMemoryTest/stackLimitGas_1023.json");
        fn mem64kb_single_byte_33("tests/GeneralStateTests/stMemoryTest/mem64kb_singleByte-33.json");
        fn log4_dejavu("tests/GeneralStateTests/stMemoryTest/log4_dejavu.json");
        fn mem64kb_single_byte("tests/GeneralStateTests/stMemoryTest/mem64kb_singleByte.json");
        fn call_data_copy_offset("tests/GeneralStateTests/stMemoryTest/callDataCopyOffset.json");
        fn mem64kb_single_byte__1("tests/GeneralStateTests/stMemoryTest/mem64kb_singleByte+1.json");
        fn calldatacopy_dejavu2("tests/GeneralStateTests/stMemoryTest/calldatacopy_dejavu2.json");
        fn mem32kb__32("tests/GeneralStateTests/stMemoryTest/mem32kb+32.json");
        fn mem64kb__32("tests/GeneralStateTests/stMemoryTest/mem64kb+32.json");
        fn mem32kb_single_byte__33("tests/GeneralStateTests/stMemoryTest/mem32kb_singleByte+33.json");
        fn stack_limit_gas_1024("tests/GeneralStateTests/stMemoryTest/stackLimitGas_1024.json");
        fn stack_limit_push32_1023("tests/GeneralStateTests/stMemoryTest/stackLimitPush32_1023.json");
        fn mem32kb_single_byte_1("tests/GeneralStateTests/stMemoryTest/mem32kb_singleByte-1.json");
        fn mem64kb_single_byte__32("tests/GeneralStateTests/stMemoryTest/mem64kb_singleByte+32.json");
        fn mem64kb_single_byte__33("tests/GeneralStateTests/stMemoryTest/mem64kb_singleByte+33.json");
        fn mem64kb_single_byte_1("tests/GeneralStateTests/stMemoryTest/mem64kb_singleByte-1.json");
        fn mem31b_single_byte("tests/GeneralStateTests/stMemoryTest/mem31b_singleByte.json");
        fn stack_limit_gas_1025("tests/GeneralStateTests/stMemoryTest/stackLimitGas_1025.json");
        fn mem64kb__33("tests/GeneralStateTests/stMemoryTest/mem64kb+33.json");
        fn mem32kb_single_byte__32("tests/GeneralStateTests/stMemoryTest/mem32kb_singleByte+32.json");
        fn mem32kb__33("tests/GeneralStateTests/stMemoryTest/mem32kb+33.json");
        fn mem32kb_single_byte("tests/GeneralStateTests/stMemoryTest/mem32kb_singleByte.json");
        fn log1_dejavu("tests/GeneralStateTests/stMemoryTest/log1_dejavu.json");
        fn mem32kb_single_byte__1("tests/GeneralStateTests/stMemoryTest/mem32kb_singleByte+1.json");
        fn mload_dejavu("tests/GeneralStateTests/stMemoryTest/mload_dejavu.json");
        fn mem64kb("tests/GeneralStateTests/stMemoryTest/mem64kb.json");
        fn mload16bit_bound("tests/GeneralStateTests/stMemoryTest/mload16bitBound.json");
        fn mem_return("tests/GeneralStateTests/stMemoryTest/memReturn.json");
        fn mem0b_single_byte("tests/GeneralStateTests/stMemoryTest/mem0b_singleByte.json");
        fn mem32kb__1("tests/GeneralStateTests/stMemoryTest/mem32kb+1.json");
        fn stack_limit_push31_1023("tests/GeneralStateTests/stMemoryTest/stackLimitPush31_1023.json");
        fn mem32kb_single_byte__31("tests/GeneralStateTests/stMemoryTest/mem32kb_singleByte+31.json");
        fn mem64kb__31("tests/GeneralStateTests/stMemoryTest/mem64kb+31.json");
        fn mem64kb_single_byte__31("tests/GeneralStateTests/stMemoryTest/mem64kb_singleByte+31.json");
        fn log3_dejavu("tests/GeneralStateTests/stMemoryTest/log3_dejavu.json");
        fn buffer_src_offset("tests/GeneralStateTests/stMemoryTest/bufferSrcOffset.json");
        fn mem32b_single_byte("tests/GeneralStateTests/stMemoryTest/mem32b_singleByte.json");
        fn mem32kb_1("tests/GeneralStateTests/stMemoryTest/mem32kb-1.json");
        fn mem32kb__31("tests/GeneralStateTests/stMemoryTest/mem32kb+31.json");
        fn mem32kb_single_byte_31("tests/GeneralStateTests/stMemoryTest/mem32kb_singleByte-31.json");
        fn mem_copy_self("tests/GeneralStateTests/stMemoryTest/memCopySelf.json");
        fn stack_limit_push31_1025("tests/GeneralStateTests/stMemoryTest/stackLimitPush31_1025.json");
        fn mem32kb_31("tests/GeneralStateTests/stMemoryTest/mem32kb-31.json");
        fn code_copy_offset("tests/GeneralStateTests/stMemoryTest/codeCopyOffset.json");
        fn extcodecopy_dejavu("tests/GeneralStateTests/stMemoryTest/extcodecopy_dejavu.json");
        fn mem64kb_single_byte_31("tests/GeneralStateTests/stMemoryTest/mem64kb_singleByte-31.json");
        fn mem33b_single_byte("tests/GeneralStateTests/stMemoryTest/mem33b_singleByte.json");
        fn stack_limit_push31_1024("tests/GeneralStateTests/stMemoryTest/stackLimitPush31_1024.json");
        fn calldatacopy_dejavu("tests/GeneralStateTests/stMemoryTest/calldatacopy_dejavu.json");
        fn mem64kb_31("tests/GeneralStateTests/stMemoryTest/mem64kb-31.json");
        fn oog("tests/GeneralStateTests/stMemoryTest/oog.json");
        fn mstore_dejavu("tests/GeneralStateTests/stMemoryTest/mstore_dejavu.json");
    }
}

mod st_e_i_p3607 {
    define_tests! {

        // --- ALL PASS ---
        fn transaction_colliding_with_non_empty_account_calls_itself("tests/GeneralStateTests/stEIP3607/transactionCollidingWithNonEmptyAccount_callsItself.json");
        fn transaction_colliding_with_non_empty_account_init("tests/GeneralStateTests/stEIP3607/transactionCollidingWithNonEmptyAccount_init.json");
        fn transaction_colliding_with_non_empty_account_send_paris("tests/GeneralStateTests/stEIP3607/transactionCollidingWithNonEmptyAccount_send_Paris.json");
        fn transaction_colliding_with_non_empty_account_send("tests/GeneralStateTests/stEIP3607/transactionCollidingWithNonEmptyAccount_send.json");
        fn transaction_colliding_with_non_empty_account_init_paris("tests/GeneralStateTests/stEIP3607/transactionCollidingWithNonEmptyAccount_init_Paris.json");
        fn transaction_colliding_with_non_empty_account_calls("tests/GeneralStateTests/stEIP3607/transactionCollidingWithNonEmptyAccount_calls.json");
        fn init_colliding_with_non_empty_account("tests/GeneralStateTests/stEIP3607/initCollidingWithNonEmptyAccount.json");
    }
}

mod st_non_zero_calls_test {
    define_tests! {

        // --- ALL PASS ---
        fn non_zero_value_s_ui_c_id_e_to_empty("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_SUICIDE_ToEmpty.json");
        fn non_zero_value_transaction_c_a_l_l_to_non_non_zero_balance("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_TransactionCALL_ToNonNonZeroBalance.json");
        fn non_zero_value_transaction_c_a_l_lwith_data_to_one_storage_key("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_TransactionCALLwithData_ToOneStorageKey.json");
        fn non_zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_one_storage_key_paris("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_DELEGATECALL_ToOneStorageKey_Paris.json");
        fn non_zero_value_c_a_l_l_c_o_d_e_to_one_storage_key("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_CALLCODE_ToOneStorageKey.json");
        fn non_zero_value_c_a_l_l("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_CALL.json");
        fn non_zero_value_transaction_c_a_l_l_to_empty_paris("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_TransactionCALL_ToEmpty_Paris.json");
        fn non_zero_value_transaction_c_a_l_lwith_data_to_empty("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_TransactionCALLwithData_ToEmpty.json");
        fn non_zero_value_c_a_l_l_c_o_d_e_to_empty_paris("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_CALLCODE_ToEmpty_Paris.json");
        fn non_zero_value_transaction_c_a_l_l_to_one_storage_key("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_TransactionCALL_ToOneStorageKey.json");
        fn non_zero_value_c_a_l_l_c_o_d_e("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_CALLCODE.json");
        fn non_zero_value_s_ui_c_id_e_to_one_storage_key("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_SUICIDE_ToOneStorageKey.json");
        fn non_zero_value_transaction_c_a_l_lwith_data("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_TransactionCALLwithData.json");
        fn non_zero_value_s_ui_c_id_e("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_SUICIDE.json");
        fn non_zero_value_c_a_l_l_to_non_non_zero_balance("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_CALL_ToNonNonZeroBalance.json");
        fn non_zero_value_s_ui_c_id_e_to_one_storage_key_paris("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_SUICIDE_ToOneStorageKey_Paris.json");
        fn non_zero_value_transaction_c_a_l_l("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_TransactionCALL.json");
        fn non_zero_value_s_ui_c_id_e_to_empty_paris("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_SUICIDE_ToEmpty_Paris.json");
        fn non_zero_value_d_e_l_e_g_a_t_e_c_a_l_l("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_DELEGATECALL.json");
        fn non_zero_value_c_a_l_l_to_empty_paris("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_CALL_ToEmpty_Paris.json");
        fn non_zero_value_transaction_c_a_l_l_to_one_storage_key_paris("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_TransactionCALL_ToOneStorageKey_Paris.json");
        fn non_zero_value_c_a_l_l_c_o_d_e_to_empty("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_CALLCODE_ToEmpty.json");
        fn non_zero_value_transaction_c_a_l_lwith_data_to_one_storage_key_paris("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_TransactionCALLwithData_ToOneStorageKey_Paris.json");
        fn non_zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_one_storage_key("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_DELEGATECALL_ToOneStorageKey.json");
        fn non_zero_value_transaction_c_a_l_lwith_data_to_non_non_zero_balance("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_TransactionCALLwithData_ToNonNonZeroBalance.json");
        fn non_zero_value_c_a_l_l_c_o_d_e_to_non_non_zero_balance("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_CALLCODE_ToNonNonZeroBalance.json");
        fn non_zero_value_c_a_l_l_to_empty("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_CALL_ToEmpty.json");
        fn non_zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_empty("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_DELEGATECALL_ToEmpty.json");
        fn non_zero_value_c_a_l_l_to_one_storage_key("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_CALL_ToOneStorageKey.json");
        fn non_zero_value_transaction_c_a_l_l_to_empty("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_TransactionCALL_ToEmpty.json");
        fn non_zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_non_non_zero_balance("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_DELEGATECALL_ToNonNonZeroBalance.json");
        fn non_zero_value_transaction_c_a_l_lwith_data_to_empty_paris("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_TransactionCALLwithData_ToEmpty_Paris.json");
        fn non_zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_empty_paris("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_DELEGATECALL_ToEmpty_Paris.json");
        fn non_zero_value_c_a_l_l_c_o_d_e_to_one_storage_key_paris("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_CALLCODE_ToOneStorageKey_Paris.json");
        fn non_zero_value_s_ui_c_id_e_to_non_non_zero_balance("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_SUICIDE_ToNonNonZeroBalance.json");
        fn non_zero_value_c_a_l_l_to_one_storage_key_paris("tests/GeneralStateTests/stNonZeroCallsTest/NonZeroValue_CALL_ToOneStorageKey_Paris.json");
    }
}

mod st_code_size_limit {
    define_tests! {

        // --- ALL PASS ---
        fn codesize_init("tests/GeneralStateTests/stCodeSizeLimit/codesizeInit.json");
        fn codesize_valid("tests/GeneralStateTests/stCodeSizeLimit/codesizeValid.json");
        fn codesize_o_o_g_invalid_size("tests/GeneralStateTests/stCodeSizeLimit/codesizeOOGInvalidSize.json");
        fn create_code_size_limit("tests/GeneralStateTests/stCodeSizeLimit/createCodeSizeLimit.json");
        fn create2_code_size_limit("tests/GeneralStateTests/stCodeSizeLimit/create2CodeSizeLimit.json");
    }
}

mod st_system_operations_test {
    define_tests! {

        // --- ALL PASS ---
        fn suicide_address("tests/GeneralStateTests/stSystemOperationsTest/suicideAddress.json");
        fn call_recursive_bomb_log2("tests/GeneralStateTests/stSystemOperationsTest/CallRecursiveBombLog2.json");
        fn call_to_return1("tests/GeneralStateTests/stSystemOperationsTest/CallToReturn1.json");
        fn call_to_name_registrator_address_too_big_left("tests/GeneralStateTests/stSystemOperationsTest/CallToNameRegistratorAddressTooBigLeft.json");
        fn a_b_acalls_suicide0("tests/GeneralStateTests/stSystemOperationsTest/ABAcallsSuicide0.json");
        fn callcode_to_return1("tests/GeneralStateTests/stSystemOperationsTest/callcodeToReturn1.json");
        fn callcode_to0("tests/GeneralStateTests/stSystemOperationsTest/callcodeTo0.json");
        fn suicide_caller("tests/GeneralStateTests/stSystemOperationsTest/suicideCaller.json");
        fn multi_selfdestruct("tests/GeneralStateTests/stSystemOperationsTest/multiSelfdestruct.json");
        fn return0("tests/GeneralStateTests/stSystemOperationsTest/return0.json");
        fn suicide_caller_addres_too_big_left("tests/GeneralStateTests/stSystemOperationsTest/suicideCallerAddresTooBigLeft.json");
        fn create_with_invalid_opcode("tests/GeneralStateTests/stSystemOperationsTest/createWithInvalidOpcode.json");
        fn extcodecopy("tests/GeneralStateTests/stSystemOperationsTest/extcodecopy.json");
        fn balance_input_address_too_big("tests/GeneralStateTests/stSystemOperationsTest/balanceInputAddressTooBig.json");
        fn return1("tests/GeneralStateTests/stSystemOperationsTest/return1.json");
        fn create_name_registrator_zero_mem2("tests/GeneralStateTests/stSystemOperationsTest/createNameRegistratorZeroMem2.json");
        fn create_name_registrator_o_o_g_mem_expansion_o_o_v("tests/GeneralStateTests/stSystemOperationsTest/createNameRegistratorOOG_MemExpansionOOV.json");
        fn callcode_to_name_registrator0("tests/GeneralStateTests/stSystemOperationsTest/callcodeToNameRegistrator0.json");
        fn a_b_acalls_suicide1("tests/GeneralStateTests/stSystemOperationsTest/ABAcallsSuicide1.json");
        fn suicide_send_ether_post_death("tests/GeneralStateTests/stSystemOperationsTest/suicideSendEtherPostDeath.json");
        fn callcode_to_name_registrator_zero_mem_expanion("tests/GeneralStateTests/stSystemOperationsTest/callcodeToNameRegistratorZeroMemExpanion.json");
        fn call_recursive_bomb2("tests/GeneralStateTests/stSystemOperationsTest/CallRecursiveBomb2.json");
        fn call_to_name_registrator_zeor_size_mem_expansion("tests/GeneralStateTests/stSystemOperationsTest/CallToNameRegistratorZeorSizeMemExpansion.json");
        fn create_name_registrator_zero_mem("tests/GeneralStateTests/stSystemOperationsTest/createNameRegistratorZeroMem.json");
        fn a_b_acalls0("tests/GeneralStateTests/stSystemOperationsTest/ABAcalls0.json");
        fn call_to_name_registrator_address_too_big_right("tests/GeneralStateTests/stSystemOperationsTest/CallToNameRegistratorAddressTooBigRight.json");
        fn suicide_caller_addres_too_big_right("tests/GeneralStateTests/stSystemOperationsTest/suicideCallerAddresTooBigRight.json");
        fn create_name_registrator_zero_mem_expansion("tests/GeneralStateTests/stSystemOperationsTest/createNameRegistratorZeroMemExpansion.json");
        fn a_b_acalls1("tests/GeneralStateTests/stSystemOperationsTest/ABAcalls1.json");
        fn create_name_registrator_value_too_high("tests/GeneralStateTests/stSystemOperationsTest/createNameRegistratorValueTooHigh.json");
        fn call_recursive_bomb0_o_o_g_at_max_call_depth("tests/GeneralStateTests/stSystemOperationsTest/CallRecursiveBomb0_OOG_atMaxCallDepth.json");
        fn create_hash_collision("tests/GeneralStateTests/stSystemOperationsTest/CreateHashCollision.json");
        fn call_recursive_bomb3("tests/GeneralStateTests/stSystemOperationsTest/CallRecursiveBomb3.json");
        fn suicide_origin("tests/GeneralStateTests/stSystemOperationsTest/suicideOrigin.json");
        fn caller_account_balance("tests/GeneralStateTests/stSystemOperationsTest/callerAccountBalance.json");
        fn create_name_registrator("tests/GeneralStateTests/stSystemOperationsTest/createNameRegistrator.json");
        fn post_to_return1("tests/GeneralStateTests/stSystemOperationsTest/PostToReturn1.json");
        fn call_to_name_registrator_too_much_memory2("tests/GeneralStateTests/stSystemOperationsTest/CallToNameRegistratorTooMuchMemory2.json");
        fn suicide_send_ether_to_me("tests/GeneralStateTests/stSystemOperationsTest/suicideSendEtherToMe.json");
        fn call_to_name_registrator0("tests/GeneralStateTests/stSystemOperationsTest/CallToNameRegistrator0.json");
        fn call_to_return1_for_dynamic_jump1("tests/GeneralStateTests/stSystemOperationsTest/CallToReturn1ForDynamicJump1.json");
        fn call_to_name_registrator_too_much_memory1("tests/GeneralStateTests/stSystemOperationsTest/CallToNameRegistratorTooMuchMemory1.json");
        fn a_b_acalls2("tests/GeneralStateTests/stSystemOperationsTest/ABAcalls2.json");
        fn callcode_to_name_registrator_addres_too_big_left("tests/GeneralStateTests/stSystemOperationsTest/callcodeToNameRegistratorAddresTooBigLeft.json");
        fn double_selfdestruct_touch_paris("tests/GeneralStateTests/stSystemOperationsTest/doubleSelfdestructTouch_Paris.json");
        fn call_recursive_bomb0("tests/GeneralStateTests/stSystemOperationsTest/CallRecursiveBomb0.json");
        fn test_name_registrator("tests/GeneralStateTests/stSystemOperationsTest/TestNameRegistrator.json");
        fn call_recursive_bomb1("tests/GeneralStateTests/stSystemOperationsTest/CallRecursiveBomb1.json");
        fn call_to_name_registrator_out_of_gas("tests/GeneralStateTests/stSystemOperationsTest/CallToNameRegistratorOutOfGas.json");
        fn a_b_acalls3("tests/GeneralStateTests/stSystemOperationsTest/ABAcalls3.json");
        fn callcode_to_name_registrator_addres_too_big_right("tests/GeneralStateTests/stSystemOperationsTest/callcodeToNameRegistratorAddresTooBigRight.json");
        fn call_to_name_registrator_too_much_memory0("tests/GeneralStateTests/stSystemOperationsTest/CallToNameRegistratorTooMuchMemory0.json");
        fn call_to_return1_for_dynamic_jump0("tests/GeneralStateTests/stSystemOperationsTest/CallToReturn1ForDynamicJump0.json");
        fn call_to_name_registrator_not_much_memory1("tests/GeneralStateTests/stSystemOperationsTest/CallToNameRegistratorNotMuchMemory1.json");
        fn suicide_not_existing_account("tests/GeneralStateTests/stSystemOperationsTest/suicideNotExistingAccount.json");
        fn create_name_registrator_out_of_memory_bonds1("tests/GeneralStateTests/stSystemOperationsTest/createNameRegistratorOutOfMemoryBonds1.json");
        fn call10("tests/GeneralStateTests/stSystemOperationsTest/Call10.json");
        fn return2("tests/GeneralStateTests/stSystemOperationsTest/return2.json");
        fn double_selfdestruct_touch("tests/GeneralStateTests/stSystemOperationsTest/doubleSelfdestructTouch.json");
        fn call_recursive_bomb_log("tests/GeneralStateTests/stSystemOperationsTest/CallRecursiveBombLog.json");
        fn double_selfdestruct_test("tests/GeneralStateTests/stSystemOperationsTest/doubleSelfdestructTest.json");
        fn current_account_balance("tests/GeneralStateTests/stSystemOperationsTest/currentAccountBalance.json");
        fn call_to_name_registrator_mem_o_o_g_and_insufficient_balance("tests/GeneralStateTests/stSystemOperationsTest/CallToNameRegistratorMemOOGAndInsufficientBalance.json");
        fn call_value("tests/GeneralStateTests/stSystemOperationsTest/callValue.json");
        fn callto_return2("tests/GeneralStateTests/stSystemOperationsTest/CalltoReturn2.json");
        fn test_random_test("tests/GeneralStateTests/stSystemOperationsTest/testRandomTest.json");
        fn create_name_registrator_out_of_memory_bonds0("tests/GeneralStateTests/stSystemOperationsTest/createNameRegistratorOutOfMemoryBonds0.json");
        fn call_to_name_registrator_not_much_memory0("tests/GeneralStateTests/stSystemOperationsTest/CallToNameRegistratorNotMuchMemory0.json");
    }
}

mod st_e_i_p1559 {
    define_tests! {

        // --- ALL PASS ---
        fn tip_too_high("tests/GeneralStateTests/stEIP1559/tipTooHigh.json");
        fn sender_balance("tests/GeneralStateTests/stEIP1559/senderBalance.json");
        fn type_two_berlin("tests/GeneralStateTests/stEIP1559/typeTwoBerlin.json");
        fn transaction_intinsic_bug("tests/GeneralStateTests/stEIP1559/transactionIntinsicBug.json");
        fn low_gas_limit("tests/GeneralStateTests/stEIP1559/lowGasLimit.json");
        fn base_fee_diff_places("tests/GeneralStateTests/stEIP1559/baseFeeDiffPlaces.json");
        fn out_of_funds_old_types("tests/GeneralStateTests/stEIP1559/outOfFundsOldTypes.json");
        fn intrinsic("tests/GeneralStateTests/stEIP1559/intrinsic.json");
        fn low_fee_cap("tests/GeneralStateTests/stEIP1559/lowFeeCap.json");
        fn out_of_funds("tests/GeneralStateTests/stEIP1559/outOfFunds.json");
        fn low_gas_price_old_types("tests/GeneralStateTests/stEIP1559/lowGasPriceOldTypes.json");
        fn val_causes_o_o_f("tests/GeneralStateTests/stEIP1559/valCausesOOF.json");
        fn gas_price_diff_places("tests/GeneralStateTests/stEIP1559/gasPriceDiffPlaces.json");
        fn transaction_intinsic_bug_paris("tests/GeneralStateTests/stEIP1559/transactionIntinsicBug_Paris.json");
    }
}

mod st_homestead_specific {
    define_tests! {

        // --- ALL PASS ---
        fn create_contract_via_contract("tests/GeneralStateTests/stHomesteadSpecific/createContractViaContract.json");
        fn create_contract_via_transaction_cost53000("tests/GeneralStateTests/stHomesteadSpecific/createContractViaTransactionCost53000.json");
        fn create_contract_via_contract_o_o_g_init_code("tests/GeneralStateTests/stHomesteadSpecific/createContractViaContractOOGInitCode.json");
        fn contract_creation_o_o_gdont_leave_empty_contract_via_transaction("tests/GeneralStateTests/stHomesteadSpecific/contractCreationOOGdontLeaveEmptyContractViaTransaction.json");
        fn contract_creation_o_o_gdont_leave_empty_contract("tests/GeneralStateTests/stHomesteadSpecific/contractCreationOOGdontLeaveEmptyContract.json");
    }
}

mod st_create2 {
    define_tests! {

        // --- ALL PASS --- (2 tests fail on standard REVM)
        fn create2_o_o_gafter_init_code("tests/GeneralStateTests/stCreate2/Create2OOGafterInitCode.json");
        fn create2_o_o_gafter_init_code_returndata("tests/GeneralStateTests/stCreate2/Create2OOGafterInitCodeReturndata.json");
        fn c_r_e_a_t_e2_bounds2("tests/GeneralStateTests/stCreate2/CREATE2_Bounds2.json");
        fn c_r_e_a_t_e2_high_nonce_delegatecall("tests/GeneralStateTests/stCreate2/CREATE2_HighNonceDelegatecall.json");
        fn create2_o_o_gafter_init_code_returndata_size("tests/GeneralStateTests/stCreate2/Create2OOGafterInitCodeReturndataSize.json");
        // fn revert_in_create_in_init_create2_paris("tests/GeneralStateTests/stCreate2/RevertInCreateInInitCreate2Paris.json");
        fn revert_depth_create_address_collision_berlin("tests/GeneralStateTests/stCreate2/RevertDepthCreateAddressCollisionBerlin.json");
        fn c_r_e_a_t_e2_bounds3("tests/GeneralStateTests/stCreate2/CREATE2_Bounds3.json");
        fn create2collision_nonce("tests/GeneralStateTests/stCreate2/create2collisionNonce.json");
        fn revert_opcode_create("tests/GeneralStateTests/stCreate2/RevertOpcodeCreate.json");
        fn returndatasize_following_successful_create("tests/GeneralStateTests/stCreate2/returndatasize_following_successful_create.json");
        fn returndatacopy_following_revert_in_create("tests/GeneralStateTests/stCreate2/returndatacopy_following_revert_in_create.json");
        fn create2collision_code2("tests/GeneralStateTests/stCreate2/create2collisionCode2.json");
        fn create2_o_o_g_from_call_refunds("tests/GeneralStateTests/stCreate2/Create2OOGFromCallRefunds.json");
        fn create2_o_o_gafter_init_code_revert2("tests/GeneralStateTests/stCreate2/Create2OOGafterInitCodeRevert2.json");
        fn create2call_precompiles("tests/GeneralStateTests/stCreate2/create2callPrecompiles.json");
        fn create2check_fields_in_initcode("tests/GeneralStateTests/stCreate2/create2checkFieldsInInitcode.json");
        fn revert_depth_create_address_collision("tests/GeneralStateTests/stCreate2/RevertDepthCreateAddressCollision.json");
        fn call_outsize_then_create2_successful_then_returndatasize("tests/GeneralStateTests/stCreate2/call_outsize_then_create2_successful_then_returndatasize.json");
        fn create2collision_balance("tests/GeneralStateTests/stCreate2/create2collisionBalance.json");
        fn returndatacopy_after_failing_create("tests/GeneralStateTests/stCreate2/returndatacopy_afterFailing_create.json");
        fn c_r_e_a_t_e2_high_nonce_minus1("tests/GeneralStateTests/stCreate2/CREATE2_HighNonceMinus1.json");
        fn returndatacopy_following_successful_create("tests/GeneralStateTests/stCreate2/returndatacopy_following_successful_create.json");
        fn create_message_reverted_o_o_g_in_init2("tests/GeneralStateTests/stCreate2/CreateMessageRevertedOOGInInit2.json");
        fn create_message_reverted("tests/GeneralStateTests/stCreate2/CreateMessageReverted.json");
        fn create2collision_selfdestructed_o_o_g("tests/GeneralStateTests/stCreate2/create2collisionSelfdestructedOOG.json");
        fn create2_o_o_gafter_init_code_revert("tests/GeneralStateTests/stCreate2/Create2OOGafterInitCodeRevert.json");
        fn returndatacopy_following_create("tests/GeneralStateTests/stCreate2/returndatacopy_following_create.json");
        fn call_then_create2_successful_then_returndatasize("tests/GeneralStateTests/stCreate2/call_then_create2_successful_then_returndatasize.json");
        fn create2_o_o_gafter_init_code_returndata2("tests/GeneralStateTests/stCreate2/Create2OOGafterInitCodeReturndata2.json");
        fn create2collision_selfdestructed("tests/GeneralStateTests/stCreate2/create2collisionSelfdestructed.json");
        fn c_r_e_a_t_e2_first_byte_loop("tests/GeneralStateTests/stCreate2/CREATE2_FirstByte_loop.json");
        fn c_r_e_a_t_e2_bounds("tests/GeneralStateTests/stCreate2/CREATE2_Bounds.json");
        fn c_r_e_a_t_e2_high_nonce("tests/GeneralStateTests/stCreate2/CREATE2_HighNonce.json");
        fn create2collision_storage("tests/GeneralStateTests/stCreate2/create2collisionStorage.json");
        fn create2_on_depth1023("tests/GeneralStateTests/stCreate2/Create2OnDepth1023.json");
        fn c_r_e_a_t_e2_contract_suicide_during_init_then_store_then_return("tests/GeneralStateTests/stCreate2/CREATE2_ContractSuicideDuringInit_ThenStoreThenReturn.json");
        fn create2_recursive("tests/GeneralStateTests/stCreate2/Create2Recursive.json");
        fn create2_smart_init_code("tests/GeneralStateTests/stCreate2/create2SmartInitCode.json");
        fn create2_o_o_gafter_init_code_returndata3("tests/GeneralStateTests/stCreate2/Create2OOGafterInitCodeReturndata3.json");
        fn returndatacopy_0_0_following_successful_create("tests/GeneralStateTests/stCreate2/returndatacopy_0_0_following_successful_create.json");
        fn revert_in_create_in_init_create2("tests/GeneralStateTests/stCreate2/RevertInCreateInInitCreate2.json");
        fn create2collision_code("tests/GeneralStateTests/stCreate2/create2collisionCode.json");
        fn create2_init_codes("tests/GeneralStateTests/stCreate2/create2InitCodes.json");
        fn create2_on_depth1024("tests/GeneralStateTests/stCreate2/Create2OnDepth1024.json");
        fn create_message_reverted_o_o_g_in_init("tests/GeneralStateTests/stCreate2/CreateMessageRevertedOOGInInit.json");
        fn revert_opcode_in_create_returns_create2("tests/GeneralStateTests/stCreate2/RevertOpcodeInCreateReturnsCreate2.json");
        fn create2no_cash("tests/GeneralStateTests/stCreate2/create2noCash.json");
        fn revert_depth_create2_o_o_g_berlin("tests/GeneralStateTests/stCreate2/RevertDepthCreate2OOGBerlin.json");
        fn revert_depth_create2_o_o_g("tests/GeneralStateTests/stCreate2/RevertDepthCreate2OOG.json");
        // fn create2collision_storage_paris("tests/GeneralStateTests/stCreate2/create2collisionStorageParis.json");
        fn c_r_e_a_t_e2_suicide("tests/GeneralStateTests/stCreate2/CREATE2_Suicide.json");
        fn create2collision_selfdestructed_revert("tests/GeneralStateTests/stCreate2/create2collisionSelfdestructedRevert.json");
        fn create2collision_selfdestructed2("tests/GeneralStateTests/stCreate2/create2collisionSelfdestructed2.json");
    }
}

mod st_static_flag_enabled {
    define_tests! {

        // --- MOST PASS --- (3 tests fail)
        fn call_with_zero_value_to_precompile_from_called_contract("tests/GeneralStateTests/stStaticFlagEnabled/CallWithZeroValueToPrecompileFromCalledContract.json");
        fn call_with_n_o_t_zero_value_to_precompile_from_called_contract("tests/GeneralStateTests/stStaticFlagEnabled/CallWithNOTZeroValueToPrecompileFromCalledContract.json");
        fn call_with_zero_value_to_precompile_from_transaction("tests/GeneralStateTests/stStaticFlagEnabled/CallWithZeroValueToPrecompileFromTransaction.json");
        fn delegatecall_to_precompile_from_transaction("tests/GeneralStateTests/stStaticFlagEnabled/DelegatecallToPrecompileFromTransaction.json");
        fn delegatecall_to_precompile_from_called_contract("tests/GeneralStateTests/stStaticFlagEnabled/DelegatecallToPrecompileFromCalledContract.json");
        fn call_with_zero_value_to_precompile_from_contract_initialization("tests/GeneralStateTests/stStaticFlagEnabled/CallWithZeroValueToPrecompileFromContractInitialization.json");
        fn callcode_to_precompile_from_transaction("tests/GeneralStateTests/stStaticFlagEnabled/CallcodeToPrecompileFromTransaction.json");
        fn callcode_to_precompile_from_called_contract("tests/GeneralStateTests/stStaticFlagEnabled/CallcodeToPrecompileFromCalledContract.json");
        fn callcode_to_precompile_from_contract_initialization("tests/GeneralStateTests/stStaticFlagEnabled/CallcodeToPrecompileFromContractInitialization.json");
        fn call_with_n_o_t_zero_value_to_precompile_from_contract_initialization("tests/GeneralStateTests/stStaticFlagEnabled/CallWithNOTZeroValueToPrecompileFromContractInitialization.json");
        fn call_with_n_o_t_zero_value_to_precompile_from_transaction("tests/GeneralStateTests/stStaticFlagEnabled/CallWithNOTZeroValueToPrecompileFromTransaction.json");
        fn delegatecall_to_precompile_from_contract_initialization("tests/GeneralStateTests/stStaticFlagEnabled/DelegatecallToPrecompileFromContractInitialization.json");
        fn staticcall_for_precompiles_issue683("tests/GeneralStateTests/stStaticFlagEnabled/StaticcallForPrecompilesIssue683.json");
    }
}

mod st_call_delegate_codes_homestead {
    define_tests! {

        // --- MOST PASS ---
        fn callcallcode_01("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcode_01.json");
        fn callcallcallcode_001("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcallcode_001.json");
        fn callcodecallcall_100_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcall_100_OOGE.json");
        fn callcallcallcode_001_suicide_end("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcallcode_001_SuicideEnd.json");
        fn callcodecallcode_11("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcode_11.json");
        fn callcodecallcodecall_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcodecall_ABCB_RECURSIVE.json");
        fn callcodecallcallcode_101("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcallcode_101.json");
        fn callcodecallcall_100_suicide_end("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcall_100_SuicideEnd.json");
        fn callcodecallcodecallcode_111_o_o_g_m_before("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcodecallcode_111_OOGMBefore.json");
        fn callcallcallcode_001_o_o_g_m_after("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcallcode_001_OOGMAfter.json");
        fn callcodecallcodecallcode_111_o_o_g_m_after("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcodecallcode_111_OOGMAfter.json");
        fn callcodecallcodecall_110_suicide_end("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcodecall_110_SuicideEnd.json");
        fn callcallcallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcallcode_ABCB_RECURSIVE.json");
        fn callcallcode_01_suicide_end("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcode_01_SuicideEnd.json");
        fn callcallcodecall_010_o_o_g_m_before("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcodecall_010_OOGMBefore.json");
        fn callcodecallcodecall_110_suicide_middle("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcodecall_110_SuicideMiddle.json");
        fn callcodecall_10_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecall_10_OOGE.json");
        fn callcodecallcall_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcall_ABCB_RECURSIVE.json");
        fn callcallcodecallcode_011("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcodecallcode_011.json");
        fn callcallcodecallcode_011_o_o_g_m_before("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcodecallcode_011_OOGMBefore.json");
        fn callcallcodecall_010_o_o_g_m_after("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcodecall_010_OOGMAfter.json");
        fn callcodecallcodecallcode_111_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcodecallcode_111_OOGE.json");
        fn callcodecallcall_100("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcall_100.json");
        fn callcodecallcallcode_101_o_o_g_m_before("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcallcode_101_OOGMBefore.json");
        fn callcodecallcallcode_101_suicide_middle("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcallcode_101_SuicideMiddle.json");
        fn callcodecall_10_suicide_end("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecall_10_SuicideEnd.json");
        fn callcallcodecall_010_suicide_end("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcodecall_010_SuicideEnd.json");
        fn callcodecallcodecall_110_o_o_g_m_after("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcodecall_110_OOGMAfter.json");
        fn callcallcodecallcode_011_o_o_g_m_after("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcodecallcode_011_OOGMAfter.json");
        fn callcallcodecall_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcodecall_ABCB_RECURSIVE.json");
        fn callcallcodecallcode_011_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcodecallcode_011_OOGE.json");
        fn callcodecallcall_100_o_o_g_m_after("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcall_100_OOGMAfter.json");
        fn callcallcodecallcode_011_suicide_end("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcodecallcode_011_SuicideEnd.json");
        fn callcodecallcodecallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcodecallcode_ABCB_RECURSIVE.json");
        fn callcodecallcallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcallcode_ABCB_RECURSIVE.json");
        fn callcodecallcallcode_101_suicide_end("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcallcode_101_SuicideEnd.json");
        fn callcodecallcodecallcode_111("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcodecallcode_111.json");
        fn callcallcodecall_010_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcodecall_010_OOGE.json");
        fn callcallcallcode_001_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcallcode_001_OOGE.json");
        fn callcodecallcallcode_101_o_o_g_m_after("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcallcode_101_OOGMAfter.json");
        fn callcodecallcall_100_suicide_middle("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcall_100_SuicideMiddle.json");
        fn callcodecallcodecall_110("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcodecall_110.json");
        fn callcallcodecallcode_011_suicide_middle("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcodecallcode_011_SuicideMiddle.json");
        fn callcodecallcode_11_suicide_end("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcode_11_SuicideEnd.json");
        fn callcallcallcode_001_suicide_middle("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcallcode_001_SuicideMiddle.json");
        fn callcallcallcode_001_o_o_g_m_before("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcallcode_001_OOGMBefore.json");
        fn callcodecallcodecallcode_111_suicide_middle("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcodecallcode_111_SuicideMiddle.json");
        fn callcodecallcall_100_o_o_g_m_before("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcall_100_OOGMBefore.json");
        fn callcallcodecall_010_suicide_middle("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcodecall_010_SuicideMiddle.json");
        fn callcodecallcode_11_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcode_11_OOGE.json");
        fn callcodecallcodecallcode_111_suicide_end("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcodecallcode_111_SuicideEnd.json");
        fn callcallcode_01_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcode_01_OOGE.json");
        fn callcodecallcodecall_110_o_o_g_m_before("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcodecall_110_OOGMBefore.json");
        fn callcallcodecallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcodecallcode_ABCB_RECURSIVE.json");
        fn callcallcodecall_010("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcallcodecall_010.json");
        fn callcodecall_10("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecall_10.json");
        fn callcodecallcodecall_110_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcodecall_110_OOGE.json");
        fn callcodecallcallcode_101_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesHomestead/callcodecallcallcode_101_OOGE.json");
    }
}

mod st_s_store_test {
    define_tests! {

        // --- ALL PASS --- (1 test fails on REVM)
        fn sstore_xto_xto_y("tests/GeneralStateTests/stSStoreTest/sstore_XtoXtoY.json");
        fn init_collision("tests/GeneralStateTests/stSStoreTest/InitCollision.json");
        fn sstore_xto_x("tests/GeneralStateTests/stSStoreTest/sstore_XtoX.json");
        fn sstore_xto_yto_x("tests/GeneralStateTests/stSStoreTest/sstore_XtoYtoX.json");
        fn sstore_0to_xto_x("tests/GeneralStateTests/stSStoreTest/sstore_0toXtoX.json");
        fn sstore_0to_xto_y("tests/GeneralStateTests/stSStoreTest/sstore_0toXtoY.json");
        fn sstore_xto_yto_y("tests/GeneralStateTests/stSStoreTest/sstore_XtoYtoY.json");
        fn sstore_xto_y("tests/GeneralStateTests/stSStoreTest/sstore_XtoY.json");
        fn sstore_xto_xto_x("tests/GeneralStateTests/stSStoreTest/sstore_XtoXtoX.json");
        fn sstore_0to_xto0("tests/GeneralStateTests/stSStoreTest/sstore_0toXto0.json");
        fn sstore_xto_yto0("tests/GeneralStateTests/stSStoreTest/sstore_XtoYto0.json");
        fn sstore_gas_left("tests/GeneralStateTests/stSStoreTest/sstore_gasLeft.json");
        fn sstore_xto0("tests/GeneralStateTests/stSStoreTest/sstore_Xto0.json");
        fn sstore_gas("tests/GeneralStateTests/stSStoreTest/sstoreGas.json");
        fn sstore_xto_xto0("tests/GeneralStateTests/stSStoreTest/sstore_XtoXto0.json");
        fn sstore_call_to_self_sub_refund_below_zero("tests/GeneralStateTests/stSStoreTest/SstoreCallToSelfSubRefundBelowZero.json");
        fn sstore_0to0("tests/GeneralStateTests/stSStoreTest/sstore_0to0.json");
        // fn init_collision_paris("tests/GeneralStateTests/stSStoreTest/InitCollisionParis.json");
        fn sstore_0to0to0("tests/GeneralStateTests/stSStoreTest/sstore_0to0to0.json");
        fn sstore_xto0to0("tests/GeneralStateTests/stSStoreTest/sstore_Xto0to0.json");
        fn sstore_xto0to_xto0("tests/GeneralStateTests/stSStoreTest/sstore_Xto0toXto0.json");
        fn sstore_xto_yto_z("tests/GeneralStateTests/stSStoreTest/sstore_XtoYtoZ.json");
        fn sstore_0to0to_x("tests/GeneralStateTests/stSStoreTest/sstore_0to0toX.json");
        fn sstore_xto0to_y("tests/GeneralStateTests/stSStoreTest/sstore_Xto0toY.json");
        fn sstore_0to_x("tests/GeneralStateTests/stSStoreTest/sstore_0toX.json");
        fn sstore_xto0to_x("tests/GeneralStateTests/stSStoreTest/sstore_Xto0toX.json");
        fn sstore_change_from_external_call_in_init_code("tests/GeneralStateTests/stSStoreTest/sstore_changeFromExternalCallInInitCode.json");
        fn init_collision_non_zero_nonce("tests/GeneralStateTests/stSStoreTest/InitCollisionNonZeroNonce.json");
        fn sstore_0to_xto0to_x("tests/GeneralStateTests/stSStoreTest/sstore_0toXto0toX.json");
    }
}

mod st_call_delegate_codes_call_code_homestead {
    define_tests! {

        // --- MOST PASS --- (3 test fail)
        fn callcallcode_01("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcode_01.json");
        fn callcallcallcode_001("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcallcode_001.json");
        fn callcodecallcall_100_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcall_100_OOGE.json");
        fn callcallcallcode_001_suicide_end("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcallcode_001_SuicideEnd.json");
        fn callcodecallcode_11("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcode_11.json");
        fn callcodecallcodecall_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcodecall_ABCB_RECURSIVE.json");
        fn callcodecallcallcode_101("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcallcode_101.json");
        fn callcodecallcall_100_suicide_end("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcall_100_SuicideEnd.json");
        fn callcodecallcodecallcode_111_o_o_g_m_before("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcodecallcode_111_OOGMBefore.json");
        fn callcallcallcode_001_o_o_g_m_after("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcallcode_001_OOGMAfter.json");
        fn callcodecallcodecallcode_111_o_o_g_m_after("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcodecallcode_111_OOGMAfter.json");
        fn callcodecallcodecall_110_suicide_end("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcodecall_110_SuicideEnd.json");
        fn callcallcallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcallcode_ABCB_RECURSIVE.json");
        fn callcallcode_01_suicide_end("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcode_01_SuicideEnd.json");
        fn callcallcodecall_010_o_o_g_m_before("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcodecall_010_OOGMBefore.json");
        fn callcodecallcodecall_110_suicide_middle("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcodecall_110_SuicideMiddle.json");
        fn callcodecall_10_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecall_10_OOGE.json");
        fn callcodecallcall_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcall_ABCB_RECURSIVE.json");
        fn callcallcodecallcode_011("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcodecallcode_011.json");
        fn callcallcodecallcode_011_o_o_g_m_before("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcodecallcode_011_OOGMBefore.json");
        fn callcallcodecall_010_o_o_g_m_after("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcodecall_010_OOGMAfter.json");
        fn callcodecallcodecallcode_111_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcodecallcode_111_OOGE.json");
        fn callcodecallcall_100("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcall_100.json");
        fn callcodecallcallcode_101_o_o_g_m_before("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcallcode_101_OOGMBefore.json");
        fn callcodecallcallcode_101_suicide_middle("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcallcode_101_SuicideMiddle.json");
        fn callcodecall_10_suicide_end("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecall_10_SuicideEnd.json");
        fn callcallcodecall_010_suicide_end("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcodecall_010_SuicideEnd.json");
        fn callcodecallcodecall_110_o_o_g_m_after("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcodecall_110_OOGMAfter.json");
        fn callcallcodecallcode_011_o_o_g_m_after("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcodecallcode_011_OOGMAfter.json");
        fn callcallcodecall_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcodecall_ABCB_RECURSIVE.json");
        fn callcallcodecallcode_011_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcodecallcode_011_OOGE.json");
        fn callcodecallcall_100_o_o_g_m_after("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcall_100_OOGMAfter.json");
        fn callcallcodecallcode_011_suicide_end("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcodecallcode_011_SuicideEnd.json");
        fn callcodecallcodecallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcodecallcode_ABCB_RECURSIVE.json");
        fn callcodecallcallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcallcode_ABCB_RECURSIVE.json");
        fn callcodecallcallcode_101_suicide_end("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcallcode_101_SuicideEnd.json");
        fn callcodecallcodecallcode_111("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcodecallcode_111.json");
        fn callcallcodecall_010_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcodecall_010_OOGE.json");
        fn callcallcallcode_001_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcallcode_001_OOGE.json");
        fn callcodecallcallcode_101_o_o_g_m_after("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcallcode_101_OOGMAfter.json");
        fn callcodecallcall_100_suicide_middle("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcall_100_SuicideMiddle.json");
        fn callcodecallcodecall_110("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcodecall_110.json");
        fn callcallcodecallcode_011_suicide_middle("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcodecallcode_011_SuicideMiddle.json");
        fn callcodecallcode_11_suicide_end("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcode_11_SuicideEnd.json");
        fn callcallcallcode_001_suicide_middle("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcallcode_001_SuicideMiddle.json");
        fn callcallcallcode_001_o_o_g_m_before("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcallcode_001_OOGMBefore.json");
        fn callcodecallcodecallcode_111_suicide_middle("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcodecallcode_111_SuicideMiddle.json");
        fn callcodecallcall_100_o_o_g_m_before("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcall_100_OOGMBefore.json");
        fn callcallcodecall_010_suicide_middle("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcodecall_010_SuicideMiddle.json");
        fn callcodecallcode_11_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcode_11_OOGE.json");
        fn callcodecallcodecallcode_111_suicide_end("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcodecallcode_111_SuicideEnd.json");
        fn callcallcode_01_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcode_01_OOGE.json");
        fn callcodecallcodecall_110_o_o_g_m_before("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcodecall_110_OOGMBefore.json");
        fn callcallcodecallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcodecallcode_ABCB_RECURSIVE.json");
        fn callcallcodecall_010("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcallcodecall_010.json");
        fn callcodecall_10("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecall_10.json");
        fn callcodecallcodecall_110_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcodecall_110_OOGE.json");
        fn callcodecallcallcode_101_o_o_g_e("tests/GeneralStateTests/stCallDelegateCodesCallCodeHomestead/callcodecallcallcode_101_OOGE.json");
    }
}

mod st_delegatecall_test_homestead {
    define_tests! {

        // --- MOST PASS ---
        fn call1024_balance_too_low("tests/GeneralStateTests/stDelegatecallTestHomestead/Call1024BalanceTooLow.json");
        fn callcode_lose_gas_o_o_g("tests/GeneralStateTests/stDelegatecallTestHomestead/CallcodeLoseGasOOG.json");
        fn deleagate_call_after_value_transfer("tests/GeneralStateTests/stDelegatecallTestHomestead/deleagateCallAfterValueTransfer.json");
        fn delegatecall1024("tests/GeneralStateTests/stDelegatecallTestHomestead/Delegatecall1024.json");
        fn delegatecall_and_o_o_gat_tx_level("tests/GeneralStateTests/stDelegatecallTestHomestead/delegatecallAndOOGatTxLevel.json");
        fn call_lose_gas_o_o_g("tests/GeneralStateTests/stDelegatecallTestHomestead/CallLoseGasOOG.json");
        fn delegatecall_in_initcode_to_empty_contract("tests/GeneralStateTests/stDelegatecallTestHomestead/delegatecallInInitcodeToEmptyContract.json");
        fn delegatecall_sender_check("tests/GeneralStateTests/stDelegatecallTestHomestead/delegatecallSenderCheck.json");
        fn call1024_o_o_g("tests/GeneralStateTests/stDelegatecallTestHomestead/Call1024OOG.json");
        fn call_with_high_value_and_gas_o_o_g("tests/GeneralStateTests/stDelegatecallTestHomestead/callWithHighValueAndGasOOG.json");
        fn call_output1("tests/GeneralStateTests/stDelegatecallTestHomestead/callOutput1.json");
        fn delegatecall_in_initcode_to_existing_contract("tests/GeneralStateTests/stDelegatecallTestHomestead/delegatecallInInitcodeToExistingContract.json");
        fn call1024_pre_calls("tests/GeneralStateTests/stDelegatecallTestHomestead/Call1024PreCalls.json");
        fn delegatecall_emptycontract("tests/GeneralStateTests/stDelegatecallTestHomestead/delegatecallEmptycontract.json");
        fn call_output2("tests/GeneralStateTests/stDelegatecallTestHomestead/callOutput2.json");
        fn call_recursive_bomb_pre_call("tests/GeneralStateTests/stDelegatecallTestHomestead/CallRecursiveBombPreCall.json");
        fn delegatecall_value_check("tests/GeneralStateTests/stDelegatecallTestHomestead/delegatecallValueCheck.json");
        fn delegatecall_basic("tests/GeneralStateTests/stDelegatecallTestHomestead/delegatecallBasic.json");
        fn call_output3("tests/GeneralStateTests/stDelegatecallTestHomestead/callOutput3.json");
        fn delegatecall1024_o_o_g("tests/GeneralStateTests/stDelegatecallTestHomestead/Delegatecall1024OOG.json");
        fn callcode_with_high_value_and_gas_o_o_g("tests/GeneralStateTests/stDelegatecallTestHomestead/callcodeWithHighValueAndGasOOG.json");
        fn callcode_output3("tests/GeneralStateTests/stDelegatecallTestHomestead/callcodeOutput3.json");
        fn call_output3partial("tests/GeneralStateTests/stDelegatecallTestHomestead/callOutput3partial.json");
        fn delegatecode_dynamic_code("tests/GeneralStateTests/stDelegatecallTestHomestead/delegatecodeDynamicCode.json");
        fn delegatecall_in_initcode_to_existing_contract_o_o_g("tests/GeneralStateTests/stDelegatecallTestHomestead/delegatecallInInitcodeToExistingContractOOG.json");
        fn delegatecode_dynamic_code2_self_call("tests/GeneralStateTests/stDelegatecallTestHomestead/delegatecodeDynamicCode2SelfCall.json");
        fn delegatecall_o_o_gin_call("tests/GeneralStateTests/stDelegatecallTestHomestead/delegatecallOOGinCall.json");
        fn call_output3partial_fail("tests/GeneralStateTests/stDelegatecallTestHomestead/callOutput3partialFail.json");
    }
}

mod st_e_i_p150_specific {
    define_tests! {

        // --- ALL PASS ---
        fn delegate_call_on_e_i_p("tests/GeneralStateTests/stEIP150Specific/DelegateCallOnEIP.json");
        fn call_goes_o_o_g_on_second_level2("tests/GeneralStateTests/stEIP150Specific/CallGoesOOGOnSecondLevel2.json");
        fn suicide_to_existing_contract("tests/GeneralStateTests/stEIP150Specific/SuicideToExistingContract.json");
        fn new_gas_price_for_codes("tests/GeneralStateTests/stEIP150Specific/NewGasPriceForCodes.json");
        fn transaction64_rule_d64e0("tests/GeneralStateTests/stEIP150Specific/Transaction64Rule_d64e0.json");
        fn create_and_gas_inside_create("tests/GeneralStateTests/stEIP150Specific/CreateAndGasInsideCreate.json");
        fn call_goes_o_o_g_on_second_level("tests/GeneralStateTests/stEIP150Specific/CallGoesOOGOnSecondLevel.json");
        fn execute_call_that_ask_fore_gas_then_trabsaction_has("tests/GeneralStateTests/stEIP150Specific/ExecuteCallThatAskForeGasThenTrabsactionHas.json");
        fn transaction64_rule_d64p1("tests/GeneralStateTests/stEIP150Specific/Transaction64Rule_d64p1.json");
        fn transaction64_rule_integer_boundaries("tests/GeneralStateTests/stEIP150Specific/Transaction64Rule_integerBoundaries.json");
        fn suicide_to_not_existing_contract("tests/GeneralStateTests/stEIP150Specific/SuicideToNotExistingContract.json");
        fn transaction64_rule_d64m1("tests/GeneralStateTests/stEIP150Specific/Transaction64Rule_d64m1.json");
        fn call_and_callcode_consume_more_gas_then_transaction_has("tests/GeneralStateTests/stEIP150Specific/CallAndCallcodeConsumeMoreGasThenTransactionHas.json");
        fn call_ask_more_gas_on_depth2_then_transaction_has("tests/GeneralStateTests/stEIP150Specific/CallAskMoreGasOnDepth2ThenTransactionHas.json");
    }
}

mod st_e_i_p150single_code_gas_prices {
    define_tests! {

        // --- ALL PASS ---
        fn gas_cost_return("tests/GeneralStateTests/stEIP150singleCodeGasPrices/gasCostReturn.json");
        fn raw_ext_code_size_gas("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawExtCodeSizeGas.json");
        fn eip2929("tests/GeneralStateTests/stEIP150singleCodeGasPrices/eip2929.json");
        fn gas_cost_jump("tests/GeneralStateTests/stEIP150singleCodeGasPrices/gasCostJump.json");
        fn raw_call_code_gas_value_transfer("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallCodeGasValueTransfer.json");
        fn gas_cost_exp("tests/GeneralStateTests/stEIP150singleCodeGasPrices/gasCostExp.json");
        fn raw_call_code_gas("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallCodeGas.json");
        fn raw_delegate_call_gas_ask("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawDelegateCallGasAsk.json");
        fn raw_call_code_gas_value_transfer_memory("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallCodeGasValueTransferMemory.json");
        fn raw_delegate_call_gas("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawDelegateCallGas.json");
        fn raw_create_fail_gas_value_transfer("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCreateFailGasValueTransfer.json");
        fn raw_call_gas("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallGas.json");
        fn eip2929_o_o_g("tests/GeneralStateTests/stEIP150singleCodeGasPrices/eip2929OOG.json");
        fn gas_cost_memory("tests/GeneralStateTests/stEIP150singleCodeGasPrices/gasCostMemory.json");
        fn raw_call_code_gas_value_transfer_ask("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallCodeGasValueTransferAsk.json");
        fn raw_call_code_gas_memory("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallCodeGasMemory.json");
        fn raw_call_memory_gas("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallMemoryGas.json");
        fn raw_call_gas_value_transfer_memory_ask("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallGasValueTransferMemoryAsk.json");
        fn raw_balance_gas("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawBalanceGas.json");
        fn raw_ext_code_copy_memory_gas("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawExtCodeCopyMemoryGas.json");
        fn gas_cost_mem_seg("tests/GeneralStateTests/stEIP150singleCodeGasPrices/gasCostMemSeg.json");
        fn eip2929_ff("tests/GeneralStateTests/stEIP150singleCodeGasPrices/eip2929-ff.json");
        fn raw_call_gas_value_transfer_ask("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallGasValueTransferAsk.json");
        fn raw_call_gas_value_transfer("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallGasValueTransfer.json");
        fn raw_call_gas_value_transfer_memory("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallGasValueTransferMemory.json");
        fn raw_create_gas_memory("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCreateGasMemory.json");
        fn raw_call_gas_ask("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallGasAsk.json");
        fn raw_create_gas("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCreateGas.json");
        fn raw_create_fail_gas_value_transfer2("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCreateFailGasValueTransfer2.json");
        fn raw_call_code_gas_value_transfer_memory_ask("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallCodeGasValueTransferMemoryAsk.json");
        fn raw_create_gas_value_transfer("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCreateGasValueTransfer.json");
        fn raw_create_gas_value_transfer_memory("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCreateGasValueTransferMemory.json");
        fn raw_ext_code_copy_gas("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawExtCodeCopyGas.json");
        fn raw_call_memory_gas_ask("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallMemoryGasAsk.json");
        fn gas_cost_berlin("tests/GeneralStateTests/stEIP150singleCodeGasPrices/gasCostBerlin.json");
        fn gas_cost("tests/GeneralStateTests/stEIP150singleCodeGasPrices/gasCost.json");
        fn raw_call_code_gas_ask("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallCodeGasAsk.json");
        fn raw_delegate_call_gas_memory_ask("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawDelegateCallGasMemoryAsk.json");
        fn raw_delegate_call_gas_memory("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawDelegateCallGasMemory.json");
        fn raw_call_code_gas_memory_ask("tests/GeneralStateTests/stEIP150singleCodeGasPrices/RawCallCodeGasMemoryAsk.json");
    }
}

mod st_zero_knowledge2 {
    define_tests! {

        // --- ALL PASS ---
        fn ecmul_0_0_340282366920938463463374607431768211456_28000_80("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_340282366920938463463374607431768211456_28000_80.json");
        fn ecadd_69_19274124_124124_25000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_6-9_19274124-124124_25000_128.json");
        fn ecmul_0_0_0_21000_64("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_0_21000_64.json");
        fn ecmul_0_0_1_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_1_21000_128.json");
        fn ecmul_0_0_340282366920938463463374607431768211456_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_340282366920938463463374607431768211456_28000_96.json");
        fn ecmul_0_0_0_28000_40("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_0_28000_40.json");
        fn ecadd_00_0_0_21000_0("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_0-0_21000_0.json");
        fn ecmul_0_0_5616_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_5616_21000_96.json");
        fn ecmul_0_0_9_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_9_21000_128.json");
        fn ecmul_0_3_2_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_2_21000_96.json");
        fn ecadd_12_0_0_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1-2_0-0_21000_128.json");
        fn ecmul_0_0_1_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_1_28000_96.json");
        fn ecmul_1_2_0_28000_80("tests/GeneralStateTests/stZeroKnowledge2/ecmul_1-2_0_28000_80.json");
        fn ecmul_0_3_9935_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_9935_28000_96.json");
        fn ecmul_1_2_0_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_1-2_0_28000_96.json");
        fn ecmul_0_0_1_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_1_28000_128.json");
        fn ecmul_0_0_9_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_9_28000_128.json");
        fn ecmul_0_3_9_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_9_28000_96.json");
        fn ecadd_00_0_0_25000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_0-0_25000_128.json");
        fn ecadd_03_1_2_25000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-3_1-2_25000_128.json");
        fn ecadd_00_1_3_25000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_1-3_25000_128.json");
        fn ecadd_00_1_2_25000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_1-2_25000_128.json");
        fn ecadd_00_0_0_25000_64("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_0-0_25000_64.json");
        fn ecmul_0_0_2_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_2_28000_96.json");
        fn ecadd_12_1_2_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1-2_1-2_21000_128.json");
        fn ecmul_0_3_0_28000_64("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_0_28000_64.json");
        fn ecmul_0_0_0_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_0_28000_128.json");
        fn ecmul_0_0_9935_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_9935_28000_96.json");
        fn ecmul_0_3_5616_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_5616_21000_96.json");
        fn ecmul_0_0_9_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_9_21000_96.json");
        fn ecadd_13_0_0_25000_80("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1-3_0-0_25000_80.json");
        fn ecmul_0_0_0_21000_0("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_0_21000_0.json");
        fn ecmul_0_3_1_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_1_21000_96.json");
        fn ecmul_1_2_2_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_1-2_2_21000_128.json");
        fn ecmul_0_0_0_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_0_21000_128.json");
        fn ecmul_0_0_2_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_2_21000_128.json");
        fn ecmul_0_0_340282366920938463463374607431768211456_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_340282366920938463463374607431768211456_21000_128.json");
        fn ecmul_1_2_0_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_1-2_0_21000_128.json");
        fn ecmul_1_2_2_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_1-2_2_21000_96.json");
        fn ecadd_12_0_0_25000_192("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1-2_0-0_25000_192.json");
        fn ecmul_0_3_0_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_0_28000_96.json");
        fn ecmul_0_3_0_28000_80("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_0_28000_80.json");
        fn ecadd_00_0_0_21000_192("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_0-0_21000_192.json");
        fn ecmul_1_2_0_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_1-2_0_28000_128.json");
        fn ecmul_0_3_5617_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_5617_28000_96.json");
        fn ecmul_0_0_340282366920938463463374607431768211456_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_340282366920938463463374607431768211456_28000_128.json");
        fn ecmul_0_0_2_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_2_28000_128.json");
        fn ecadd_00_0_0_25000_80("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_0-0_25000_80.json");
        fn ecadd_11453932_1145_4651_21000_192("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1145-3932_1145-4651_21000_192.json");
        fn ecadd_00_1_2_21000_192("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_1-2_21000_192.json");
        fn ecmul_1_2_1_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_1-2_1_28000_128.json");
        fn ecmul_0_0_5617_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_5617_28000_96.json");
        fn ecadd_12_1_2_25000_192("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1-2_1-2_25000_192.json");
        fn ecmul_0_3_340282366920938463463374607431768211456_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_340282366920938463463374607431768211456_21000_96.json");
        fn ecadd_12_0_0_25000_64("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1-2_0-0_25000_64.json");
        fn ecmul_0_3_340282366920938463463374607431768211456_21000_80("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_340282366920938463463374607431768211456_21000_80.json");
        fn ecadd_13_0_0_25000_80_paris("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1-3_0-0_25000_80_Paris.json");
        fn ecmul_1_2_0_28000_64("tests/GeneralStateTests/stZeroKnowledge2/ecmul_1-2_0_28000_64.json");
        fn ecmul_1_2_1_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_1-2_1_21000_128.json");
        fn ecadd_11453932_2969_1336_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1145-3932_2969-1336_21000_128.json");
        fn ecmul_0_0_0_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_0_21000_96.json");
        fn ecmul_1_2_1_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_1-2_1_21000_96.json");
        fn ecmul_0_0_0_21000_80("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_0_21000_80.json");
        fn ecmul_0_0_340282366920938463463374607431768211456_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_340282366920938463463374607431768211456_21000_96.json");
        fn ecmul_0_0_0_21000_40("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_0_21000_40.json");
        fn ecmul_0_0_5616_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_5616_21000_128.json");
        fn ecmul_0_0_0_28000_64("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_0_28000_64.json");
        fn ecmul_0_3_340282366920938463463374607431768211456_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_340282366920938463463374607431768211456_21000_128.json");
        fn ecmul_0_0_0_28000_0("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_0_28000_0.json");
        fn ecmul_0_0_340282366920938463463374607431768211456_21000_80("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_340282366920938463463374607431768211456_21000_80.json");
        fn ecmul_0_3_2_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_2_28000_128.json");
        fn ecmul_0_3_5616_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_5616_28000_128.json");
        fn ecmul_0_3_2_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_2_28000_96.json");
        fn ecmul_0_0_5616_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_5616_28000_96.json");
        fn ecadd_69_19274124_124124_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_6-9_19274124-124124_21000_128.json");
        fn ecmul_1_2_0_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_1-2_0_21000_96.json");
        fn ecmul_0_3_9935_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_9935_21000_96.json");
        fn ecmul_1_2_0_21000_80("tests/GeneralStateTests/stZeroKnowledge2/ecmul_1-2_0_21000_80.json");
        fn ecmul_0_3_340282366920938463463374607431768211456_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_340282366920938463463374607431768211456_28000_128.json");
        fn ecadd_00_0_0_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_0-0_21000_128.json");
        fn ecmul_0_0_1_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_1_21000_96.json");
        fn ecadd_03_1_2_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-3_1-2_21000_128.json");
        fn ecadd_00_1_3_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_1-3_21000_128.json");
        fn ecmul_0_0_5616_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_5616_28000_128.json");
        fn ecmul_0_3_5616_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_5616_21000_128.json");
        fn ecmul_0_3_2_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_2_21000_128.json");
        fn ecadd_12_0_0_25000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1-2_0-0_25000_128.json");
        fn ecmul_0_3_9_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_9_21000_96.json");
        fn ecmul_0_3_5617_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_5617_21000_128.json");
        fn ecadd_13_0_0_21000_80("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1-3_0-0_21000_80.json");
        fn ecadd_12_1_2_25000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1-2_1-2_25000_128.json");
        fn ecmul_0_0_2_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_2_21000_96.json");
        fn ecmul_0_0_9935_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_9935_21000_96.json");
        fn ecadd_00_0_0_25000_0("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_0-0_25000_0.json");
        fn ecmul_0_0_5617_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_5617_28000_128.json");
        fn ecadd_00_1_2_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_1-2_21000_128.json");
        fn ecmul_0_3_0_21000_64("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_0_21000_64.json");
        fn ecadd_00_0_0_21000_64("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_0-0_21000_64.json");
        fn ecmul_0_3_5617_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_5617_28000_128.json");
        fn ecmul_0_0_9_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_9_28000_96.json");
        fn ecmul_0_3_5616_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_5616_28000_96.json");
        fn ecmul_0_3_5616_28000_96_paris("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_5616_28000_96_Paris.json");
        fn ecmul_0_0_5617_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_5617_21000_128.json");
        fn ecmul_0_3_1_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_1_28000_96.json");
        fn ecmul_0_3_1_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_1_28000_128.json");
        fn ecmul_0_3_9935_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_9935_28000_128.json");
        fn ecmul_0_0_9935_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_9935_21000_128.json");
        fn ecmul_0_3_9_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_9_28000_128.json");
        fn ecadd_00_0_0_21000_80("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_0-0_21000_80.json");
        fn ecmul_0_3_0_21000_80("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_0_21000_80.json");
        fn ecmul_0_3_9935_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_9935_21000_128.json");
        fn ecadd_00_0_0_25000_192("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_0-0_25000_192.json");
        fn ecmul_0_3_0_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_0_21000_96.json");
        fn ecmul_0_3_1_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_1_21000_128.json");
        fn ecmul_0_3_9_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_9_21000_128.json");
        fn ecmul_0_0_9935_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_9935_28000_128.json");
        fn ecadd_12_0_0_21000_192("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1-2_0-0_21000_192.json");
        fn ecadd_00_0_0_21000_80_paris("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_0-0_21000_80_Paris.json");
        fn ecmul_0_3_5617_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_5617_21000_96.json");
        fn ecmul_0_0_5617_21000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_5617_21000_96.json");
        fn ecadd_12_1_2_21000_192("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1-2_1-2_21000_192.json");
        fn ecmul_1_2_0_21000_64("tests/GeneralStateTests/stZeroKnowledge2/ecmul_1-2_0_21000_64.json");
        fn ecmul_0_3_340282366920938463463374607431768211456_28000_80("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_340282366920938463463374607431768211456_28000_80.json");
        fn ecmul_0_3_0_21000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_0_21000_128.json");
        fn ecadd_00_1_2_25000_192("tests/GeneralStateTests/stZeroKnowledge2/ecadd_0-0_1-2_25000_192.json");
        fn ecadd_11453932_1145_4651_25000_192("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1145-3932_1145-4651_25000_192.json");
        fn ecmul_0_3_340282366920938463463374607431768211456_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_340282366920938463463374607431768211456_28000_96.json");
        fn ecadd_11453932_2969_1336_25000_128("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1145-3932_2969-1336_25000_128.json");
        fn ecadd_12_0_0_21000_64("tests/GeneralStateTests/stZeroKnowledge2/ecadd_1-2_0-0_21000_64.json");
        fn ecmul_0_0_0_28000_80("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_0_28000_80.json");
        fn ecmul_1_2_1_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_1-2_1_28000_96.json");
        fn ecmul_0_0_0_28000_96("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-0_0_28000_96.json");
        fn ecmul_0_3_0_28000_128("tests/GeneralStateTests/stZeroKnowledge2/ecmul_0-3_0_28000_128.json");
    }
}

mod st_log_tests {
    define_tests! {

        // --- ALL PASS ---
        fn log2_log_memsize_too_high("tests/GeneralStateTests/stLogTests/log2_logMemsizeTooHigh.json");
        fn log1_non_empty_mem("tests/GeneralStateTests/stLogTests/log1_nonEmptyMem.json");
        fn log4_log_memsize_zero("tests/GeneralStateTests/stLogTests/log4_logMemsizeZero.json");
        fn log1_empty_mem("tests/GeneralStateTests/stLogTests/log1_emptyMem.json");
        fn log0_log_memsize_zero("tests/GeneralStateTests/stLogTests/log0_logMemsizeZero.json");
        fn log2_empty_mem("tests/GeneralStateTests/stLogTests/log2_emptyMem.json");
        fn log4_non_empty_mem_log_mem_size1_log_mem_start31("tests/GeneralStateTests/stLogTests/log4_nonEmptyMem_logMemSize1_logMemStart31.json");
        fn log1_log_memsize_zero("tests/GeneralStateTests/stLogTests/log1_logMemsizeZero.json");
        fn log2_caller("tests/GeneralStateTests/stLogTests/log2_Caller.json");
        fn log1_log_memsize_too_high("tests/GeneralStateTests/stLogTests/log1_logMemsizeTooHigh.json");
        fn log2_log_memsize_zero("tests/GeneralStateTests/stLogTests/log2_logMemsizeZero.json");
        fn log2_log_mem_start_too_high("tests/GeneralStateTests/stLogTests/log2_logMemStartTooHigh.json");
        fn log3_log_memsize_zero("tests/GeneralStateTests/stLogTests/log3_logMemsizeZero.json");
        fn log2_max_topic("tests/GeneralStateTests/stLogTests/log2_MaxTopic.json");
        fn log1_log_mem_start_too_high("tests/GeneralStateTests/stLogTests/log1_logMemStartTooHigh.json");
        fn log0_non_empty_mem("tests/GeneralStateTests/stLogTests/log0_nonEmptyMem.json");
        fn log0_log_memsize_too_high("tests/GeneralStateTests/stLogTests/log0_logMemsizeTooHigh.json");
        fn log1_caller("tests/GeneralStateTests/stLogTests/log1_Caller.json");
        fn log3_non_empty_mem_log_mem_size1("tests/GeneralStateTests/stLogTests/log3_nonEmptyMem_logMemSize1.json");
        fn log0_non_empty_mem_log_mem_size1("tests/GeneralStateTests/stLogTests/log0_nonEmptyMem_logMemSize1.json");
        fn log4_caller("tests/GeneralStateTests/stLogTests/log4_Caller.json");
        fn log4_log_memsize_too_high("tests/GeneralStateTests/stLogTests/log4_logMemsizeTooHigh.json");
        fn log4_non_empty_mem("tests/GeneralStateTests/stLogTests/log4_nonEmptyMem.json");
        fn log1_max_topic("tests/GeneralStateTests/stLogTests/log1_MaxTopic.json");
        fn log3_log_memsize_too_high("tests/GeneralStateTests/stLogTests/log3_logMemsizeTooHigh.json");
        fn log3_p_c("tests/GeneralStateTests/stLogTests/log3_PC.json");
        fn log3_non_empty_mem_log_mem_size1_log_mem_start31("tests/GeneralStateTests/stLogTests/log3_nonEmptyMem_logMemSize1_logMemStart31.json");
        fn log3_log_mem_start_too_high("tests/GeneralStateTests/stLogTests/log3_logMemStartTooHigh.json");
        fn log1_non_empty_mem_log_mem_size1("tests/GeneralStateTests/stLogTests/log1_nonEmptyMem_logMemSize1.json");
        fn log4_max_topic("tests/GeneralStateTests/stLogTests/log4_MaxTopic.json");
        fn log3_caller("tests/GeneralStateTests/stLogTests/log3_Caller.json");
        fn log3_non_empty_mem("tests/GeneralStateTests/stLogTests/log3_nonEmptyMem.json");
        fn log3_max_topic("tests/GeneralStateTests/stLogTests/log3_MaxTopic.json");
        fn log1_non_empty_mem_log_mem_size1_log_mem_start31("tests/GeneralStateTests/stLogTests/log1_nonEmptyMem_logMemSize1_logMemStart31.json");
        fn log2_non_empty_mem_log_mem_size1("tests/GeneralStateTests/stLogTests/log2_nonEmptyMem_logMemSize1.json");
        fn log2_non_empty_mem_log_mem_size1_log_mem_start31("tests/GeneralStateTests/stLogTests/log2_nonEmptyMem_logMemSize1_logMemStart31.json");
        fn log3_empty_mem("tests/GeneralStateTests/stLogTests/log3_emptyMem.json");
        fn log4_log_mem_start_too_high("tests/GeneralStateTests/stLogTests/log4_logMemStartTooHigh.json");
        fn log4_non_empty_mem_log_mem_size1("tests/GeneralStateTests/stLogTests/log4_nonEmptyMem_logMemSize1.json");
        fn log2_non_empty_mem("tests/GeneralStateTests/stLogTests/log2_nonEmptyMem.json");
        fn log_in_o_o_g_call("tests/GeneralStateTests/stLogTests/logInOOG_Call.json");
        fn log4_p_c("tests/GeneralStateTests/stLogTests/log4_PC.json");
        fn log4_empty_mem("tests/GeneralStateTests/stLogTests/log4_emptyMem.json");
        fn log0_non_empty_mem_log_mem_size1_log_mem_start31("tests/GeneralStateTests/stLogTests/log0_nonEmptyMem_logMemSize1_logMemStart31.json");
        fn log0_log_mem_start_too_high("tests/GeneralStateTests/stLogTests/log0_logMemStartTooHigh.json");
        fn log0_empty_mem("tests/GeneralStateTests/stLogTests/log0_emptyMem.json");
    }
}

mod st_chain_id {
    define_tests! {

        // --- ALL PASS ---
        fn chain_id("tests/GeneralStateTests/stChainId/chainId.json");
        fn chain_id_gas_cost("tests/GeneralStateTests/stChainId/chainIdGasCost.json");
    }
}

mod st_self_balance {
    define_tests! {

        // --- ALL PASS ---
        fn self_balance_gas_cost("tests/GeneralStateTests/stSelfBalance/selfBalanceGasCost.json");
        fn diff_places("tests/GeneralStateTests/stSelfBalance/diffPlaces.json");
        fn self_balance_update("tests/GeneralStateTests/stSelfBalance/selfBalanceUpdate.json");
        fn self_balance_equals_balance("tests/GeneralStateTests/stSelfBalance/selfBalanceEqualsBalance.json");
        fn self_balance_call_types("tests/GeneralStateTests/stSelfBalance/selfBalanceCallTypes.json");
        fn self_balance("tests/GeneralStateTests/stSelfBalance/selfBalance.json");
    }
}

mod st_e_i_p158_specific {
    define_tests! {

        // --- ALL PASS ---
        fn e_x_t_c_o_d_e_s_i_z_e_to_non_existent("tests/GeneralStateTests/stEIP158Specific/EXTCODESIZE_toNonExistent.json");
        fn e_x_p_empty("tests/GeneralStateTests/stEIP158Specific/EXP_Empty.json");
        fn vitalik_transaction_test("tests/GeneralStateTests/stEIP158Specific/vitalikTransactionTest.json");
        fn call_to_empty_then_call_error_paris("tests/GeneralStateTests/stEIP158Specific/callToEmptyThenCallErrorParis.json");
        fn e_x_t_c_o_d_e_s_i_z_e_to_epmty_paris("tests/GeneralStateTests/stEIP158Specific/EXTCODESIZE_toEpmtyParis.json");
        fn c_a_l_l_zero_v_call_suicide("tests/GeneralStateTests/stEIP158Specific/CALL_ZeroVCallSuicide.json");
        fn c_a_l_l_one_v_call_suicide("tests/GeneralStateTests/stEIP158Specific/CALL_OneVCallSuicide.json");
        fn c_a_l_l_one_v_call_suicide2("tests/GeneralStateTests/stEIP158Specific/CALL_OneVCallSuicide2.json");
        fn call_to_empty_then_call_error("tests/GeneralStateTests/stEIP158Specific/callToEmptyThenCallError.json");
        fn vitalik_transaction_test_paris("tests/GeneralStateTests/stEIP158Specific/vitalikTransactionTestParis.json");
        fn e_x_t_c_o_d_e_s_i_z_e_to_epmty("tests/GeneralStateTests/stEIP158Specific/EXTCODESIZE_toEpmty.json");
    }
}

mod st_zero_calls_revert {
    define_tests! {

        // --- ALL PASS ---
        fn zero_value_c_a_l_l_to_non_zero_balance_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_CALL_ToNonZeroBalance_OOGRevert.json");
        fn zero_value_c_a_l_l_to_one_storage_key_o_o_g_revert_paris("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_CALL_ToOneStorageKey_OOGRevert_Paris.json");
        fn zero_value_c_a_l_l_c_o_d_e_to_empty_o_o_g_revert_paris("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_CALLCODE_ToEmpty_OOGRevert_Paris.json");
        fn zero_value_s_ui_c_id_e_to_one_storage_key_o_o_g_revert_paris("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_SUICIDE_ToOneStorageKey_OOGRevert_Paris.json");
        fn zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_one_storage_key_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_DELEGATECALL_ToOneStorageKey_OOGRevert.json");
        fn zero_value_s_ui_c_id_e_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_SUICIDE_OOGRevert.json");
        fn zero_value_c_a_l_l_to_empty_o_o_g_revert_paris("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_CALL_ToEmpty_OOGRevert_Paris.json");
        fn zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_empty_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_DELEGATECALL_ToEmpty_OOGRevert.json");
        fn zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_one_storage_key_o_o_g_revert_paris("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_DELEGATECALL_ToOneStorageKey_OOGRevert_Paris.json");
        fn zero_value_s_ui_c_id_e_to_one_storage_key_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_SUICIDE_ToOneStorageKey_OOGRevert.json");
        fn zero_value_s_ui_c_id_e_to_non_zero_balance_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_SUICIDE_ToNonZeroBalance_OOGRevert.json");
        fn zero_value_c_a_l_l_c_o_d_e_to_non_zero_balance_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_CALLCODE_ToNonZeroBalance_OOGRevert.json");
        fn zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_empty_o_o_g_revert_paris("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_DELEGATECALL_ToEmpty_OOGRevert_Paris.json");
        fn zero_value_c_a_l_l_c_o_d_e_to_one_storage_key_o_o_g_revert_paris("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_CALLCODE_ToOneStorageKey_OOGRevert_Paris.json");
        fn zero_value_c_a_l_l_c_o_d_e_to_one_storage_key_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_CALLCODE_ToOneStorageKey_OOGRevert.json");
        fn zero_value_s_ui_c_id_e_to_empty_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_SUICIDE_ToEmpty_OOGRevert.json");
        fn zero_value_c_a_l_l_c_o_d_e_to_empty_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_CALLCODE_ToEmpty_OOGRevert.json");
        fn zero_value_d_e_l_e_g_a_t_e_c_a_l_l_to_non_zero_balance_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_DELEGATECALL_ToNonZeroBalance_OOGRevert.json");
        fn zero_value_d_e_l_e_g_a_t_e_c_a_l_l_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_DELEGATECALL_OOGRevert.json");
        fn zero_value_s_ui_c_id_e_to_empty_o_o_g_revert_paris("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_SUICIDE_ToEmpty_OOGRevert_Paris.json");
        fn zero_value_c_a_l_l_to_one_storage_key_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_CALL_ToOneStorageKey_OOGRevert.json");
        fn zero_value_c_a_l_l_c_o_d_e_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_CALLCODE_OOGRevert.json");
        fn zero_value_c_a_l_l_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_CALL_OOGRevert.json");
        fn zero_value_c_a_l_l_to_empty_o_o_g_revert("tests/GeneralStateTests/stZeroCallsRevert/ZeroValue_CALL_ToEmpty_OOGRevert.json");
    }
}

mod st_transaction_test {
    define_tests! {

        // --- ALL PASS ---
        fn create_message_success("tests/GeneralStateTests/stTransactionTest/CreateMessageSuccess.json");
        fn internal_call_hitting_gas_limit_success("tests/GeneralStateTests/stTransactionTest/InternalCallHittingGasLimitSuccess.json");
        fn value_overflow_paris("tests/GeneralStateTests/stTransactionTest/ValueOverflowParis.json");
        fn high_gas_price_paris("tests/GeneralStateTests/stTransactionTest/HighGasPriceParis.json");
        fn suicides_and_internl_call_suicides_bonus_gas_at_call_failed("tests/GeneralStateTests/stTransactionTest/SuicidesAndInternlCallSuicidesBonusGasAtCallFailed.json");
        fn transaction_data_costs652("tests/GeneralStateTests/stTransactionTest/TransactionDataCosts652.json");
        fn overflow_gas_require2("tests/GeneralStateTests/stTransactionTest/OverflowGasRequire2.json");
        fn contract_store_clears_o_o_g("tests/GeneralStateTests/stTransactionTest/ContractStoreClearsOOG.json");
        fn internl_call_store_clears_o_o_g("tests/GeneralStateTests/stTransactionTest/InternlCallStoreClearsOOG.json");
        fn suicides_and_send_money_to_itself_ether_destroyed("tests/GeneralStateTests/stTransactionTest/SuicidesAndSendMoneyToItselfEtherDestroyed.json");
        fn transaction_to_itself("tests/GeneralStateTests/stTransactionTest/TransactionToItself.json");
        fn no_src_account_create("tests/GeneralStateTests/stTransactionTest/NoSrcAccountCreate.json");
        fn suicides_and_internl_call_suicides_bonus_gas_at_call("tests/GeneralStateTests/stTransactionTest/SuicidesAndInternlCallSuicidesBonusGasAtCall.json");
        fn no_src_account1559("tests/GeneralStateTests/stTransactionTest/NoSrcAccount1559.json");
        fn internal_call_hitting_gas_limit2("tests/GeneralStateTests/stTransactionTest/InternalCallHittingGasLimit2.json");
        fn create_message_reverted("tests/GeneralStateTests/stTransactionTest/CreateMessageReverted.json");
        fn no_src_account_create1559("tests/GeneralStateTests/stTransactionTest/NoSrcAccountCreate1559.json");
        fn store_clears_and_internl_call_store_clears_o_o_g("tests/GeneralStateTests/stTransactionTest/StoreClearsAndInternlCallStoreClearsOOG.json");
        fn internl_call_store_clears_succes("tests/GeneralStateTests/stTransactionTest/InternlCallStoreClearsSucces.json");
        fn store_gas_on_create("tests/GeneralStateTests/stTransactionTest/StoreGasOnCreate.json");
        fn contract_store_clears_success("tests/GeneralStateTests/stTransactionTest/ContractStoreClearsSuccess.json");
        fn opcodes_transaction_init("tests/GeneralStateTests/stTransactionTest/Opcodes_TransactionInit.json");
        fn high_gas_price("tests/GeneralStateTests/stTransactionTest/HighGasPrice.json");
        fn value_overflow("tests/GeneralStateTests/stTransactionTest/ValueOverflow.json");
        fn empty_transaction3("tests/GeneralStateTests/stTransactionTest/EmptyTransaction3.json");
        fn store_clears_and_internl_call_store_clears_success("tests/GeneralStateTests/stTransactionTest/StoreClearsAndInternlCallStoreClearsSuccess.json");
        fn high_gas_limit("tests/GeneralStateTests/stTransactionTest/HighGasLimit.json");
        fn no_src_account("tests/GeneralStateTests/stTransactionTest/NoSrcAccount.json");
        fn suicides_and_internl_call_suicides_success("tests/GeneralStateTests/stTransactionTest/SuicidesAndInternlCallSuicidesSuccess.json");
        fn transaction_sending_to_empty("tests/GeneralStateTests/stTransactionTest/TransactionSendingToEmpty.json");
        fn suicides_and_internl_call_suicides_o_o_g("tests/GeneralStateTests/stTransactionTest/SuicidesAndInternlCallSuicidesOOG.json");
        fn transaction_to_addressh160minus_one("tests/GeneralStateTests/stTransactionTest/TransactionToAddressh160minusOne.json");
        fn point_at_infinity_e_c_recover("tests/GeneralStateTests/stTransactionTest/PointAtInfinityECRecover.json");
        fn suicides_stop_after_suicide("tests/GeneralStateTests/stTransactionTest/SuicidesStopAfterSuicide.json");
        fn create_transaction_success("tests/GeneralStateTests/stTransactionTest/CreateTransactionSuccess.json");
        fn internal_call_hitting_gas_limit("tests/GeneralStateTests/stTransactionTest/InternalCallHittingGasLimit.json");
        fn transaction_sending_to_zero("tests/GeneralStateTests/stTransactionTest/TransactionSendingToZero.json");
    }
}

mod st_zero_knowledge {
    define_tests! {

        // --- ALL PASS ---
        fn ecpairing_empty_data_insufficient_gas("tests/GeneralStateTests/stZeroKnowledge/ecpairing_empty_data_insufficient_gas.json");
        fn ecpairing_perturb_g2_by_curve_order("tests/GeneralStateTests/stZeroKnowledge/ecpairing_perturb_g2_by_curve_order.json");
        fn ecmul_1_3_0_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_0_21000_128.json");
        fn ecmul_1_2_5616_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_5616_21000_128.json");
        fn ecmul_1_3_0_28000_80_paris("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_0_28000_80_Paris.json");
        fn ecmul_1_3_0_28000_64("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_0_28000_64.json");
        fn ecpairing_inputs("tests/GeneralStateTests/stZeroKnowledge/ecpairing_inputs.json");
        fn ecmul_1_3_5616_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_5616_21000_128.json");
        fn ecmul_1_3_5617_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_5617_28000_96.json");
        fn ecmul_1_2_5616_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_5616_28000_128.json");
        fn ecmul_1_3_0_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_0_28000_128.json");
        fn ecmul_1_3_1_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_1_21000_96.json");
        fn ecmul_1_3_5616_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_5616_28000_128.json");
        fn ecmul_7827_6598_5616_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_5616_28000_96.json");
        fn ecmul_1_3_9_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_9_28000_128.json");
        fn ecmul_1_3_5617_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_5617_28000_128.json");
        fn ecpairing_bad_length_191("tests/GeneralStateTests/stZeroKnowledge/ecpairing_bad_length_191.json");
        fn ecmul_1_2_340282366920938463463374607431768211456_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_340282366920938463463374607431768211456_21000_96.json");
        fn ecpairing_perturb_zeropoint_by_one("tests/GeneralStateTests/stZeroKnowledge/ecpairing_perturb_zeropoint_by_one.json");
        fn ecmul_1_3_2_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_2_21000_96.json");
        fn ecmul_1_3_1_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_1_28000_128.json");
        fn ecmul_1_2_9935_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_9935_21000_96.json");
        fn ecmul_1_2_5617_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_5617_28000_128.json");
        fn ecmul_1_2_340282366920938463463374607431768211456_21000_80("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_340282366920938463463374607431768211456_21000_80.json");
        fn ecpairing_one_point_with_g1_zero("tests/GeneralStateTests/stZeroKnowledge/ecpairing_one_point_with_g1_zero.json");
        fn ecmul_1_2_2_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_2_28000_128.json");
        fn point_add("tests/GeneralStateTests/stZeroKnowledge/pointAdd.json");
        fn ecmul_1_3_9_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_9_21000_128.json");
        fn ecpairing_perturb_g2_by_field_modulus("tests/GeneralStateTests/stZeroKnowledge/ecpairing_perturb_g2_by_field_modulus.json");
        fn ecmul_7827_6598_1456_21000_80("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_1456_21000_80.json");
        fn ecmul_7827_6598_0_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_0_28000_96.json");
        fn ecmul_7827_6598_1456_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_1456_21000_96.json");
        fn ecpairing_one_point_with_g2_zero_and_g1_invalid("tests/GeneralStateTests/stZeroKnowledge/ecpairing_one_point_with_g2_zero_and_g1_invalid.json");
        fn pairing_test("tests/GeneralStateTests/stZeroKnowledge/pairingTest.json");
        fn ecmul_7827_6598_0_28000_80("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_0_28000_80.json");
        fn ecmul_1_3_5617_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_5617_21000_128.json");
        fn ecpairing_two_point_match_1("tests/GeneralStateTests/stZeroKnowledge/ecpairing_two_point_match_1.json");
        fn ecmul_1_3_9_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_9_28000_96.json");
        fn ecmul_7827_6598_9935_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_9935_21000_96.json");
        fn ecmul_1_2_5617_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_5617_21000_128.json");
        fn ecpairing_two_point_fail_1("tests/GeneralStateTests/stZeroKnowledge/ecpairing_two_point_fail_1.json");
        fn ecmul_1_3_1_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_1_21000_128.json");
        fn ecmul_1_3_9935_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_9935_21000_128.json");
        fn ecmul_1_3_9935_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_9935_28000_96.json");
        fn point_mul_add2("tests/GeneralStateTests/stZeroKnowledge/pointMulAdd2.json");
        fn ecmul_7827_6598_0_28000_64("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_0_28000_64.json");
        fn ecmul_1_2_9935_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_9935_21000_128.json");
        fn point_add_trunc("tests/GeneralStateTests/stZeroKnowledge/pointAddTrunc.json");
        fn ecmul_1_3_9935_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_9935_28000_128.json");
        fn ecmul_1_2_9935_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_9935_28000_128.json");
        fn ecmul_7827_6598_1_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_1_21000_96.json");
        fn ecmul_1_2_9_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_9_28000_96.json");
        fn ecmul_1_2_5617_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_5617_21000_96.json");
        fn ecmul_1_3_340282366920938463463374607431768211456_28000_80("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_340282366920938463463374607431768211456_28000_80.json");
        fn ecmul_1_3_2_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_2_28000_128.json");
        fn ecmul_1_3_340282366920938463463374607431768211456_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_340282366920938463463374607431768211456_28000_96.json");
        fn ecmul_1_3_5616_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_5616_21000_96.json");
        fn ecmul_7827_6598_2_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_2_21000_96.json");
        fn ecmul_1_2_9_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_9_28000_128.json");
        fn ecpairing_two_point_oog("tests/GeneralStateTests/stZeroKnowledge/ecpairing_two_point_oog.json");
        fn ecmul_1_3_0_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_0_28000_96.json");
        fn ecmul_1_3_2_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_2_21000_128.json");
        fn ecpairing_three_point_fail_1("tests/GeneralStateTests/stZeroKnowledge/ecpairing_three_point_fail_1.json");
        fn ecmul_7827_6598_5617_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_5617_21000_96.json");
        fn ecmul_1_3_0_28000_80("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_0_28000_80.json");
        fn ecpairing_one_point_insufficient_gas("tests/GeneralStateTests/stZeroKnowledge/ecpairing_one_point_insufficient_gas.json");
        fn ecmul_7827_6598_9_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_9_28000_96.json");
        fn ecpairing_empty_data("tests/GeneralStateTests/stZeroKnowledge/ecpairing_empty_data.json");
        fn ecmul_1_2_9_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_9_21000_128.json");
        fn ecmul_1_3_5617_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_5617_21000_96.json");
        fn ecmul_1_2_5616_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_5616_21000_96.json");
        fn ecmul_1_3_0_21000_64("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_0_21000_64.json");
        fn ecpairing_two_point_match_4("tests/GeneralStateTests/stZeroKnowledge/ecpairing_two_point_match_4.json");
        fn ecpairing_perturb_g2_by_one("tests/GeneralStateTests/stZeroKnowledge/ecpairing_perturb_g2_by_one.json");
        fn ecmul_7827_6598_5616_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_5616_21000_96.json");
        fn ecmul_1_3_1_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_1_28000_96.json");
        fn ecmul_7827_6598_2_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_2_21000_128.json");
        fn ecmul_1_2_340282366920938463463374607431768211456_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_340282366920938463463374607431768211456_21000_128.json");
        fn ecmul_7827_6598_9935_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_9935_28000_128.json");
        fn ecpairing_three_point_match_1("tests/GeneralStateTests/stZeroKnowledge/ecpairing_three_point_match_1.json");
        fn ecmul_1_2_340282366920938463463374607431768211456_28000_80("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_340282366920938463463374607431768211456_28000_80.json");
        fn ecpairing_two_point_match_5("tests/GeneralStateTests/stZeroKnowledge/ecpairing_two_point_match_5.json");
        fn ecmul_1_2_9935_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_9935_28000_96.json");
        fn ecmul_1_3_2_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_2_28000_96.json");
        fn ecmul_1_2_340282366920938463463374607431768211456_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_340282366920938463463374607431768211456_28000_96.json");
        fn ecmul_1_2_340282366920938463463374607431768211456_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_340282366920938463463374607431768211456_28000_128.json");
        fn ecmul_7827_6598_2_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_2_28000_128.json");
        fn ecmul_7827_6598_0_21000_80("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_0_21000_80.json");
        fn ecmul_7827_6598_1456_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_1456_28000_96.json");
        fn ecmul_7827_6598_0_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_0_21000_96.json");
        fn ecmul_7827_6598_9935_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_9935_21000_128.json");
        fn point_mul_add("tests/GeneralStateTests/stZeroKnowledge/pointMulAdd.json");
        fn ecmul_7827_6598_1456_28000_80("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_1456_28000_80.json");
        fn ecmul_7827_6598_9935_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_9935_28000_96.json");
        fn ecmul_1_3_9_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_9_21000_96.json");
        fn ecmul_7827_6598_5617_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_5617_21000_128.json");
        fn ecmul_1_3_9935_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_9935_21000_96.json");
        fn ecmul_7827_6598_1456_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_1456_21000_128.json");
        fn ecpairing_two_point_match_2("tests/GeneralStateTests/stZeroKnowledge/ecpairing_two_point_match_2.json");
        fn ecmul_7827_6598_0_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_0_28000_128.json");
        fn ecmul_1_2_2_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_2_28000_96.json");
        fn ecmul_7827_6598_0_21000_64("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_0_21000_64.json");
        fn ecpairing_two_point_fail_2("tests/GeneralStateTests/stZeroKnowledge/ecpairing_two_point_fail_2.json");
        fn ecmul_7827_6598_5617_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_5617_28000_128.json");
        fn ecpairing_two_points_with_one_g2_zero("tests/GeneralStateTests/stZeroKnowledge/ecpairing_two_points_with_one_g2_zero.json");
        fn ecmul_7827_6598_0_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_0_21000_128.json");
        fn ecpairing_perturb_g2_by_field_modulus_again("tests/GeneralStateTests/stZeroKnowledge/ecpairing_perturb_g2_by_field_modulus_again.json");
        fn ecpairing_perturb_zeropoint_by_field_modulus("tests/GeneralStateTests/stZeroKnowledge/ecpairing_perturb_zeropoint_by_field_modulus.json");
        fn ecmul_7827_6598_1456_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_1456_28000_128.json");
        fn ecmul_1_2_9_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_9_21000_96.json");
        fn ecmul_7827_6598_1_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_1_28000_96.json");
        fn ecpairing_perturb_zeropoint_by_curve_order("tests/GeneralStateTests/stZeroKnowledge/ecpairing_perturb_zeropoint_by_curve_order.json");
        fn ecmul_1_3_5616_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_5616_28000_96.json");
        fn ecmul_1_3_340282366920938463463374607431768211456_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_340282366920938463463374607431768211456_21000_96.json");
        fn ecmul_7827_6598_9_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_9_21000_128.json");
        fn ecpairing_one_point_with_g2_zero("tests/GeneralStateTests/stZeroKnowledge/ecpairing_one_point_with_g2_zero.json");
        fn ecmul_1_3_340282366920938463463374607431768211456_21000_80("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_340282366920938463463374607431768211456_21000_80.json");
        fn ecpairing_one_point_not_in_subgroup("tests/GeneralStateTests/stZeroKnowledge/ecpairing_one_point_not_in_subgroup.json");
        fn ecmul_1_3_340282366920938463463374607431768211456_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_340282366920938463463374607431768211456_21000_128.json");
        fn ecmul_1_2_5617_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_5617_28000_96.json");
        fn ecpairing_bad_length_193("tests/GeneralStateTests/stZeroKnowledge/ecpairing_bad_length_193.json");
        fn ecmul_7827_6598_5616_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_5616_28000_128.json");
        fn ecmul_7827_6598_2_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_2_28000_96.json");
        fn ecmul_7827_6598_1_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_1_21000_128.json");
        fn ecmul_1_3_0_21000_80("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_0_21000_80.json");
        fn ecmul_7827_6598_5617_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_5617_28000_96.json");
        fn ecmul_7827_6598_9_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_9_28000_128.json");
        fn ecmul_1_3_340282366920938463463374607431768211456_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_340282366920938463463374607431768211456_28000_128.json");
        fn ecmul_1_3_0_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-3_0_21000_96.json");
        fn ecmul_7827_6598_5616_21000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_5616_21000_128.json");
        fn ecpairing_two_point_match_3("tests/GeneralStateTests/stZeroKnowledge/ecpairing_two_point_match_3.json");
        fn ecmul_1_2_616_28000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_1-2_616_28000_96.json");
        fn ecmul_7827_6598_9_21000_96("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_9_21000_96.json");
        fn ecmul_7827_6598_1_28000_128("tests/GeneralStateTests/stZeroKnowledge/ecmul_7827-6598_1_28000_128.json");
        fn ecpairing_one_point_fail("tests/GeneralStateTests/stZeroKnowledge/ecpairing_one_point_fail.json");
    }
}

mod st_static_call_static {
    define_tests! {

        // --- MOST PASS --- (17 tests fail)
        fn static_callcodecallcallcode_101_o_o_g_m_after_3("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_101_OOGMAfter_3.json");
        fn static_call_recursive_bomb2("tests/GeneralStateTests/stStaticCall/static_CallRecursiveBomb2.json");
        fn static_call_output3partial_fail("tests/GeneralStateTests/stStaticCall/static_callOutput3partialFail.json");
        fn static_call_ripemd160_4_gas719("tests/GeneralStateTests/stStaticCall/static_CallRipemd160_4_gas719.json");
        fn static_callcodecallcodecall_110_suicide_end("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_110_SuicideEnd.json");
        fn static_call_ecrecover_r_prefixed0("tests/GeneralStateTests/stStaticCall/static_CallEcrecoverR_prefixed0.json");
        fn static_call1024_pre_calls2("tests/GeneralStateTests/stStaticCall/static_Call1024PreCalls2.json");
        fn static_log_caller("tests/GeneralStateTests/stStaticCall/static_log_Caller.json");
        fn static_callcallcall_000_o_o_g_e("tests/GeneralStateTests/stStaticCall/static_callcallcall_000_OOGE.json");
        fn static_callcodecallcodecall_110_o_o_g_m_after_3("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_110_OOGMAfter_3.json");
        fn static_c_a_l_l_zero_v_call_suicide("tests/GeneralStateTests/stStaticCall/static_CALL_ZeroVCallSuicide.json");
        fn static_callcallcodecall_010("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_010.json");
        fn static_callcallcall_000_o_o_g_m_after2("tests/GeneralStateTests/stStaticCall/static_callcallcall_000_OOGMAfter2.json");
        fn static_callcallcodecall_a_b_c_b_r_e_c_u_r_s_i_v_e2("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_ABCB_RECURSIVE2.json");
        fn static_contract_creation_o_o_gdont_leave_empty_contract_via_transaction("tests/GeneralStateTests/stStaticCall/static_contractCreationOOGdontLeaveEmptyContractViaTransaction.json");
        fn static_log1_log_mem_start_too_high("tests/GeneralStateTests/stStaticCall/static_log1_logMemStartTooHigh.json");
        fn static_loop_calls_depth_then_revert2("tests/GeneralStateTests/stStaticCall/static_LoopCallsDepthThenRevert2.json");
        fn static_callcodecallcodecallcode_111_suicide_end("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecallcode_111_SuicideEnd.json");
        fn static_callcallcodecall_010_o_o_g_m_after_2("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_010_OOGMAfter_2.json");
        fn static_internl_call_store_clears_o_o_g("tests/GeneralStateTests/stStaticCall/static_InternlCallStoreClearsOOG.json");
        fn static_call_with_high_value("tests/GeneralStateTests/stStaticCall/static_callWithHighValue.json");
        fn static_call_sha256_1_nonzero_value("tests/GeneralStateTests/stStaticCall/static_CallSha256_1_nonzeroValue.json");
        fn static_call_identity_3("tests/GeneralStateTests/stStaticCall/static_CallIdentity_3.json");
        fn static_call50000_rip160("tests/GeneralStateTests/stStaticCall/static_Call50000_rip160.json");
        fn static_call50000bytes_contract50_1("tests/GeneralStateTests/stStaticCall/static_Call50000bytesContract50_1.json");
        fn static_callcallcodecall_010_suicide_end2("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_010_SuicideEnd2.json");
        fn static_call_recursive_bomb_pre_call2("tests/GeneralStateTests/stStaticCall/static_CallRecursiveBombPreCall2.json");
        fn staticcall_to_precompile_from_transaction("tests/GeneralStateTests/stStaticCall/StaticcallToPrecompileFromTransaction.json");
        fn static_callcallcallcode_001_o_o_g_m_after_2("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_001_OOGMAfter_2.json");
        fn static_call_sha256_3_prefix0("tests/GeneralStateTests/stStaticCall/static_CallSha256_3_prefix0.json");
        fn static_callcallcallcode_001_suicide_end2("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_001_SuicideEnd2.json");
        fn static_call_output3("tests/GeneralStateTests/stStaticCall/static_callOutput3.json");
        fn static_call10("tests/GeneralStateTests/stStaticCall/static_Call10.json");
        fn static_a_b_acalls1("tests/GeneralStateTests/stStaticCall/static_ABAcalls1.json");
        fn static_check_opcodes5("tests/GeneralStateTests/stStaticCall/static_CheckOpcodes5.json");
        fn static_call_o_o_g_additional_gas_costs1("tests/GeneralStateTests/stStaticCall/static_call_OOG_additionalGasCosts1.json");
        fn static_call_with_high_value_and_o_o_gat_tx_level("tests/GeneralStateTests/stStaticCall/static_callWithHighValueAndOOGatTxLevel.json");
        fn static_check_opcodes4("tests/GeneralStateTests/stStaticCall/static_CheckOpcodes4.json");
        fn static_check_call_cost_o_o_g("tests/GeneralStateTests/stStaticCall/static_CheckCallCostOOG.json");
        fn static_call_lose_gas_o_o_g("tests/GeneralStateTests/stStaticCall/static_CallLoseGasOOG.json");
        fn static_a_b_acalls0("tests/GeneralStateTests/stStaticCall/static_ABAcalls0.json");
        fn static_call_output2("tests/GeneralStateTests/stStaticCall/static_callOutput2.json");
        fn static_callcallcodecallcode_011_suicide_end2("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_011_SuicideEnd2.json");
        fn static_callcallcodecallcode_011_o_o_g_m_after_2("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_011_OOGMAfter_2.json");
        fn static_call_recursive_bomb_pre_call("tests/GeneralStateTests/stStaticCall/static_CallRecursiveBombPreCall.json");
        fn static_callcallcall_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stStaticCall/static_callcallcall_ABCB_RECURSIVE.json");
        fn static_call_and_callcode_consume_more_gas_then_transaction_has("tests/GeneralStateTests/stStaticCall/static_CallAndCallcodeConsumeMoreGasThenTransactionHas.json");
        fn static_callcallcallcode_001_o_o_g_m_after_3("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_001_OOGMAfter_3.json");
        fn static_call_goes_o_o_g_on_second_level2("tests/GeneralStateTests/stStaticCall/static_CallGoesOOGOnSecondLevel2.json");
        fn static_call_recursive_bomb_log("tests/GeneralStateTests/stStaticCall/static_CallRecursiveBombLog.json");
        fn static_call_identity_2("tests/GeneralStateTests/stStaticCall/static_CallIdentity_2.json");
        fn static_c_r_e_a_t_e_contract_suicide_during_init_with_value("tests/GeneralStateTests/stStaticCall/static_CREATE_ContractSuicideDuringInit_WithValue.json");
        fn static_refund_call_to_suicide_no_storage("tests/GeneralStateTests/stStaticCall/static_refund_CallToSuicideNoStorage.json");
        fn static_callcallcodecall_010_o_o_g_m_after_3("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_010_OOGMAfter_3.json");
        fn static_call_ecrecover0_gas2999("tests/GeneralStateTests/stStaticCall/static_CallEcrecover0_Gas2999.json");
        fn static_loop_calls_depth_then_revert3("tests/GeneralStateTests/stStaticCall/static_LoopCallsDepthThenRevert3.json");
        fn static_revert_opcode_calls("tests/GeneralStateTests/stStaticCall/static_RevertOpcodeCalls.json");
        fn static_callcodecall_10_2("tests/GeneralStateTests/stStaticCall/static_callcodecall_10_2.json");
        fn static_callcodecallcodecall_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_ABCB_RECURSIVE.json");
        fn static_callcodecallcodecall_110_o_o_g_e2("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_110_OOGE2.json");
        fn static_callcodecall_10_o_o_g_e_2("tests/GeneralStateTests/stStaticCall/static_callcodecall_10_OOGE_2.json");
        fn static_call_goes_o_o_g_on_second_level("tests/GeneralStateTests/stStaticCall/static_CallGoesOOGOnSecondLevel.json");
        fn static_callcodecallcodecall_110_suicide_end2("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_110_SuicideEnd2.json");
        fn static_callcodecallcall_100_o_o_g_m_before2("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100_OOGMBefore2.json");
        fn static_callcallcodecall_010_suicide_end("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_010_SuicideEnd.json");
        fn static_callcodecallcodecall_110_o_o_g_m_after_2("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_110_OOGMAfter_2.json");
        fn static_call_to_del_call_op_code_check("tests/GeneralStateTests/stStaticCall/static_callToDelCallOpCodeCheck.json");
        fn static_callcodecall_10_suicide_end2("tests/GeneralStateTests/stStaticCall/static_callcodecall_10_SuicideEnd2.json");
        fn static_r_e_t_u_r_n_bounds_o_o_g("tests/GeneralStateTests/stStaticCall/static_RETURN_BoundsOOG.json");
        fn static_call1024_pre_calls3("tests/GeneralStateTests/stStaticCall/static_Call1024PreCalls3.json");
        fn static_callcodecallcallcode_101_suicide_end2("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_101_SuicideEnd2.json");
        fn static_callcodecallcall_100_o_o_g_e("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100_OOGE.json");
        fn static_callcodecallcodecall_110_2("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_110_2.json");
        fn static_call_recursive_bomb3("tests/GeneralStateTests/stStaticCall/static_CallRecursiveBomb3.json");
        fn static_call_ecrecover_s_prefixed0("tests/GeneralStateTests/stStaticCall/static_CallEcrecoverS_prefixed0.json");
        fn static_callcallcodecall_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_ABCB_RECURSIVE.json");
        fn static_callcallcodecallcode_011_suicide_middle2("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_011_SuicideMiddle2.json");
        fn static_callcallcodecallcode_011_o_o_g_m_before2("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_011_OOGMBefore2.json");
        fn static_zero_value_c_a_l_l_o_o_g_revert("tests/GeneralStateTests/stStaticCall/static_ZeroValue_CALL_OOGRevert.json");
        fn static_log0_empty_mem("tests/GeneralStateTests/stStaticCall/static_log0_emptyMem.json");
        fn static_call_identity_5("tests/GeneralStateTests/stStaticCall/static_CallIdentity_5.json");
        fn static_call_ripemd160_1("tests/GeneralStateTests/stStaticCall/static_CallRipemd160_1.json");
        fn static_callto_return2("tests/GeneralStateTests/stStaticCall/static_CalltoReturn2.json");
        fn static_call_ecrecover1("tests/GeneralStateTests/stStaticCall/static_CallEcrecover1.json");
        fn static_call_basic("tests/GeneralStateTests/stStaticCall/static_callBasic.json");
        fn static_check_opcodes3("tests/GeneralStateTests/stStaticCall/static_CheckOpcodes3.json");
        fn static_callcodecallcodecall_110_o_o_g_m_before2("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_110_OOGMBefore2.json");
        fn static_callcodecallcall_100_o_o_g_m_before("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100_OOGMBefore.json");
        fn static_callcall_00_o_o_g_e_1("tests/GeneralStateTests/stStaticCall/static_callcall_00_OOGE_1.json");
        fn static_call50000_ecrec("tests/GeneralStateTests/stStaticCall/static_Call50000_ecrec.json");
        fn static_callcodecallcall_100_suicide_end2("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100_SuicideEnd2.json");
        fn static_return_test("tests/GeneralStateTests/stStaticCall/static_ReturnTest.json");
        fn static_call_ecrecover_check_length_wrong_v("tests/GeneralStateTests/stStaticCall/static_CallEcrecoverCheckLengthWrongV.json");
        fn static_callcodecallcall_100_o_o_g_m_after_2("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100_OOGMAfter_2.json");
        fn static_call_to_static_op_code_check("tests/GeneralStateTests/stStaticCall/static_callToStaticOpCodeCheck.json");
        fn static_call_identity_4_gas17("tests/GeneralStateTests/stStaticCall/static_CallIdentity_4_gas17.json");
        fn static_call_ecrecover_v_prefixed0("tests/GeneralStateTests/stStaticCall/static_CallEcrecoverV_prefixed0.json");
        fn static_callcallcodecall_010_o_o_g_m_after2("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_010_OOGMAfter2.json");
        fn static_callcodecallcallcode_101_o_o_g_m_before("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_101_OOGMBefore.json");
        fn static_callcodecallcallcode_101_o_o_g_m_before2("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_101_OOGMBefore2.json");
        fn static_callcallcodecallcode_011_suicide_middle("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_011_SuicideMiddle.json");
        fn static_call_to_call_code_op_code_check("tests/GeneralStateTests/stStaticCall/static_callToCallCodeOpCodeCheck.json");
        fn static_revert_depth2("tests/GeneralStateTests/stStaticCall/static_RevertDepth2.json");
        fn static_callcallcodecallcode_011_2("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_011_2.json");
        fn static_callcodecallcall_100_2("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100_2.json");
        fn static_call_create("tests/GeneralStateTests/stStaticCall/static_callCreate.json");
        fn static_callcallcall_000_suicide_end("tests/GeneralStateTests/stStaticCall/static_callcallcall_000_SuicideEnd.json");
        fn static_c_a_l_l_one_v_call_suicide("tests/GeneralStateTests/stStaticCall/static_CALL_OneVCallSuicide.json");
        fn static_callcodecallcall_100_o_o_g_m_after("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100_OOGMAfter.json");
        fn static_call_ripemd160_3_postfixed0("tests/GeneralStateTests/stStaticCall/static_CallRipemd160_3_postfixed0.json");
        fn static_callcallcallcode_001_suicide_middle2("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_001_SuicideMiddle2.json");
        fn static_callcodecallcallcode_101_suicide_middle2("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_101_SuicideMiddle2.json");
        fn static_calldelcode_01("tests/GeneralStateTests/stStaticCall/static_calldelcode_01.json");
        fn static_callcodecallcall_100_o_o_g_m_after_3("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100_OOGMAfter_3.json");
        fn static_callcodecallcodecall_110("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_110.json");
        fn static_call_sha256_1("tests/GeneralStateTests/stStaticCall/static_CallSha256_1.json");
        fn static_callcallcallcode_a_b_c_b_r_e_c_u_r_s_i_v_e2("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_ABCB_RECURSIVE2.json");
        fn static_callcall_00_suicide_end("tests/GeneralStateTests/stStaticCall/static_callcall_00_SuicideEnd.json");
        fn static_callcallcallcode_001_o_o_g_m_before("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_001_OOGMBefore.json");
        fn static_callcodecallcodecall_110_o_o_g_m_after2("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_110_OOGMAfter2.json");
        fn static_call1_m_b1024_calldepth("tests/GeneralStateTests/stStaticCall/static_Call1MB1024Calldepth.json");
        fn static_internal_call_hitting_gas_limit("tests/GeneralStateTests/stStaticCall/static_InternalCallHittingGasLimit.json");
        fn static_callcodecallcall_100_o_o_g_e2("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100_OOGE2.json");
        fn static_callcodecallcall_100_suicide_middle2("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100_SuicideMiddle2.json");
        fn static_callcallcodecallcode_011_o_o_g_m_before("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_011_OOGMBefore.json");
        fn static_check_opcodes2("tests/GeneralStateTests/stStaticCall/static_CheckOpcodes2.json");
        fn static_call50000_identity2("tests/GeneralStateTests/stStaticCall/static_Call50000_identity2.json");
        fn static_call_to_name_registrator0("tests/GeneralStateTests/stStaticCall/static_CallToNameRegistrator0.json");
        fn static_callcallcodecallcode_a_b_c_b_r_e_c_u_r_s_i_v_e2("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_ABCB_RECURSIVE2.json");
        fn static_call_recursive_bomb_log2("tests/GeneralStateTests/stStaticCall/static_CallRecursiveBombLog2.json");
        fn static_call1024_balance_too_low("tests/GeneralStateTests/stStaticCall/static_Call1024BalanceTooLow.json");
        fn static_call_ecrecover0("tests/GeneralStateTests/stStaticCall/static_CallEcrecover0.json");
        fn static_callcallcodecall_010_o_o_g_m_before2("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_010_OOGMBefore2.json");
        fn static_call_identity_4("tests/GeneralStateTests/stStaticCall/static_CallIdentity_4.json");
        fn static_callcodecallcallcode_101_2("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_101_2.json");
        fn static_callcallcode_01_o_o_g_e_2("tests/GeneralStateTests/stStaticCall/static_callcallcode_01_OOGE_2.json");
        fn static_loop_calls_then_revert("tests/GeneralStateTests/stStaticCall/static_LoopCallsThenRevert.json");
        fn static_callcallcallcode_001_o_o_g_m_before2("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_001_OOGMBefore2.json");
        fn static_callcall_00("tests/GeneralStateTests/stStaticCall/static_callcall_00.json");
        fn static_check_opcodes("tests/GeneralStateTests/stStaticCall/static_CheckOpcodes.json");
        fn static_raw_call_gas_ask("tests/GeneralStateTests/stStaticCall/static_RawCallGasAsk.json");
        fn static_call50000("tests/GeneralStateTests/stStaticCall/static_Call50000.json");
        fn static_callcodecallcall_a_b_c_b_r_e_c_u_r_s_i_v_e2("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_ABCB_RECURSIVE2.json");
        fn static_callcallcodecall_010_o_o_g_e("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_010_OOGE.json");
        fn static_callcallcallcode_001_o_o_g_e("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_001_OOGE.json");
        fn static_return_test2("tests/GeneralStateTests/stStaticCall/static_ReturnTest2.json");
        fn static_call_ecrecover3("tests/GeneralStateTests/stStaticCall/static_CallEcrecover3.json");
        fn static_callcallcode_01_suicide_end("tests/GeneralStateTests/stStaticCall/static_callcallcode_01_SuicideEnd.json");
        fn static_callcodecallcallcode_101_o_o_g_m_after("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_101_OOGMAfter.json");
        fn static_call_ripemd160_3("tests/GeneralStateTests/stStaticCall/static_CallRipemd160_3.json");
        fn static_call_output3partial("tests/GeneralStateTests/stStaticCall/static_callOutput3partial.json");
        fn static_call_contract_to_create_contract_o_o_g("tests/GeneralStateTests/stStaticCall/static_CallContractToCreateContractOOG.json");
        fn static_callcallcallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_ABCB_RECURSIVE.json");
        fn static_calldelcode_01_o_o_g_e("tests/GeneralStateTests/stStaticCall/static_calldelcode_01_OOGE.json");
        fn static_call1024_pre_calls("tests/GeneralStateTests/stStaticCall/static_Call1024PreCalls.json");
        fn static_call_value_inherit_from_call("tests/GeneralStateTests/stStaticCall/static_call_value_inherit_from_call.json");
        fn static_call_create2("tests/GeneralStateTests/stStaticCall/static_callCreate2.json");
        fn static_make_money("tests/GeneralStateTests/stStaticCall/static_makeMoney.json");
        fn static_c_r_e_a_t_e_empty_contract_and_call_it_0wei("tests/GeneralStateTests/stStaticCall/static_CREATE_EmptyContractAndCallIt_0wei.json");
        fn static_callcodecallcall_100("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100.json");
        fn static_callcallcodecallcode_011_o_o_g_e_2("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_011_OOGE_2.json");
        fn static_callcallcodecall_010_o_o_g_e_2("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_010_OOGE_2.json");
        fn static_callcodecallcall_100_suicide_middle("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100_SuicideMiddle.json");
        fn static_callcodecallcall_100_suicide_end("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100_SuicideEnd.json");
        fn static_callcallcodecall_010_o_o_g_m_after("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_010_OOGMAfter.json");
        fn static_call_o_o_g_additional_gas_costs2_paris("tests/GeneralStateTests/stStaticCall/static_call_OOG_additionalGasCosts2_Paris.json");
        fn staticcall_to_precompile_from_called_contract("tests/GeneralStateTests/stStaticCall/StaticcallToPrecompileFromCalledContract.json");
        fn static_call_sha256_2("tests/GeneralStateTests/stStaticCall/static_CallSha256_2.json");
        fn static_callcodecallcallcode_101_o_o_g_e_2("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_101_OOGE_2.json");
        fn static_callcodecallcallcode_101("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_101.json");
        fn static_callcodecallcallcode_101_suicide_end("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_101_SuicideEnd.json");
        fn static_callcodecallcallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_ABCB_RECURSIVE.json");
        fn static_callcallcallcode_001_suicide_middle("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_001_SuicideMiddle.json");
        fn static_call_ecrecover0_0input("tests/GeneralStateTests/stStaticCall/static_CallEcrecover0_0input.json");
        fn static_return50000_2("tests/GeneralStateTests/stStaticCall/static_Return50000_2.json");
        fn static_call_with_high_value_o_o_gin_call("tests/GeneralStateTests/stStaticCall/static_callWithHighValueOOGinCall.json");
        fn static_callcallcallcode_001_o_o_g_e_2("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_001_OOGE_2.json");
        fn static_call_with_high_value_and_gas_o_o_g("tests/GeneralStateTests/stStaticCall/static_callWithHighValueAndGasOOG.json");
        fn static_callcall_00_o_o_g_e_2("tests/GeneralStateTests/stStaticCall/static_callcall_00_OOGE_2.json");
        fn static_callcallcodecall_010_suicide_middle("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_010_SuicideMiddle.json");
        fn static_callcodecallcallcode_a_b_c_b_r_e_c_u_r_s_i_v_e2("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_ABCB_RECURSIVE2.json");
        fn static_call_ask_more_gas_on_depth2_then_transaction_has("tests/GeneralStateTests/stStaticCall/static_CallAskMoreGasOnDepth2ThenTransactionHas.json");
        fn static_call_sha256_3("tests/GeneralStateTests/stStaticCall/static_CallSha256_3.json");
        fn static_callcallcall_000_o_o_g_m_before("tests/GeneralStateTests/stStaticCall/static_callcallcall_000_OOGMBefore.json");
        fn static_callcallcodecallcode_011_suicide_end("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_011_SuicideEnd.json");
        fn static_callcallcodecall_010_2("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_010_2.json");
        fn static_callcall_00_o_o_g_e("tests/GeneralStateTests/stStaticCall/static_callcall_00_OOGE.json");
        fn static_call_ecrecover80("tests/GeneralStateTests/stStaticCall/static_CallEcrecover80.json");
        fn static_callcodecallcodecall_110_o_o_g_e("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_110_OOGE.json");
        fn static_callcodecallcallcode_101_o_o_g_e("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_101_OOGE.json");
        fn static_call_identity_1_nonzero_value("tests/GeneralStateTests/stStaticCall/static_CallIdentity_1_nonzeroValue.json");
        fn static_log0_non_empty_mem_log_mem_size1_log_mem_start31("tests/GeneralStateTests/stStaticCall/static_log0_nonEmptyMem_logMemSize1_logMemStart31.json");
        fn static_log1_empty_mem("tests/GeneralStateTests/stStaticCall/static_log1_emptyMem.json");
        fn static_call_output3_fail("tests/GeneralStateTests/stStaticCall/static_callOutput3Fail.json");
        fn static_callcallcallcode_001_suicide_end("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_001_SuicideEnd.json");
        fn static_zero_value_s_ui_c_id_e_o_o_g_revert("tests/GeneralStateTests/stStaticCall/static_ZeroValue_SUICIDE_OOGRevert.json");
        fn static_callcode_check_p_c("tests/GeneralStateTests/stStaticCall/static_callcode_checkPC.json");
        fn static_log0_log_mem_start_too_high("tests/GeneralStateTests/stStaticCall/static_log0_logMemStartTooHigh.json");
        fn static_refund_call_a("tests/GeneralStateTests/stStaticCall/static_refund_CallA.json");
        fn static_call_create3("tests/GeneralStateTests/stStaticCall/static_callCreate3.json");
        fn static_log0_log_memsize_zero("tests/GeneralStateTests/stStaticCall/static_log0_logMemsizeZero.json");
        fn static_callcallcall_000("tests/GeneralStateTests/stStaticCall/static_callcallcall_000.json");
        fn static_log0_log_memsize_too_high("tests/GeneralStateTests/stStaticCall/static_log0_logMemsizeTooHigh.json");
        fn static_r_e_t_u_r_n_bounds("tests/GeneralStateTests/stStaticCall/static_RETURN_Bounds.json");
        fn static_call1024_balance_too_low2("tests/GeneralStateTests/stStaticCall/static_Call1024BalanceTooLow2.json");
        fn static_call_ripemd160_2("tests/GeneralStateTests/stStaticCall/static_CallRipemd160_2.json");
        fn static_c_r_e_a_t_e_contract_suicide_during_init("tests/GeneralStateTests/stStaticCall/static_CREATE_ContractSuicideDuringInit.json");
        fn static_call50000_identity("tests/GeneralStateTests/stStaticCall/static_Call50000_identity.json");
        fn static_callcodecall_10_suicide_end("tests/GeneralStateTests/stStaticCall/static_callcodecall_10_SuicideEnd.json");
        fn static_callcodecall_10_o_o_g_e("tests/GeneralStateTests/stStaticCall/static_callcodecall_10_OOGE.json");
        fn static_call_identity_4_gas18("tests/GeneralStateTests/stStaticCall/static_CallIdentity_4_gas18.json");
        fn static_call_ecrecover2("tests/GeneralStateTests/stStaticCall/static_CallEcrecover2.json");
        fn static_call_to_return1("tests/GeneralStateTests/stStaticCall/static_CallToReturn1.json");
        fn static_log0_non_empty_mem("tests/GeneralStateTests/stStaticCall/static_log0_nonEmptyMem.json");
        fn static_call_ecrecover0_complete_return_value("tests/GeneralStateTests/stStaticCall/static_CallEcrecover0_completeReturnValue.json");
        fn static_log1_log_memsize_zero("tests/GeneralStateTests/stStaticCall/static_log1_logMemsizeZero.json");
        fn static_call_value_inherit("tests/GeneralStateTests/stStaticCall/static_call_value_inherit.json");
        fn static_c_r_e_a_t_e_contract_suicide_during_init_then_store_then_return("tests/GeneralStateTests/stStaticCall/static_CREATE_ContractSuicideDuringInit_ThenStoreThenReturn.json");
        fn static_call_ecrecover_check_length("tests/GeneralStateTests/stStaticCall/static_CallEcrecoverCheckLength.json");
        fn static_call_ecrecover0_no_gas("tests/GeneralStateTests/stStaticCall/static_CallEcrecover0_NoGas.json");
        fn static_callcodecallcodecall_a_b_c_b_r_e_c_u_r_s_i_v_e2("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_ABCB_RECURSIVE2.json");
        fn static_callcallcallcode_001_o_o_g_m_after2("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_001_OOGMAfter2.json");
        fn static_call_contract_to_create_contract_o_o_g_bonus_gas("tests/GeneralStateTests/stStaticCall/static_CallContractToCreateContractOOGBonusGas.json");
        fn static_callcodecallcodecall_110_o_o_g_m_after("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_110_OOGMAfter.json");
        fn static_callcallcodecallcode_011_o_o_g_e("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_011_OOGE.json");
        fn static_post_to_return1("tests/GeneralStateTests/stStaticCall/static_PostToReturn1.json");
        fn static_callcodecallcodecall_110_o_o_g_m_before("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_110_OOGMBefore.json");
        fn static_callcallcall_000_o_o_g_m_after("tests/GeneralStateTests/stStaticCall/static_callcallcall_000_OOGMAfter.json");
        fn static_callcodecallcodecall_110_suicide_middle("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_110_SuicideMiddle.json");
        fn static_callcallcodecallcode_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_ABCB_RECURSIVE.json");
        fn static_callcallcodecallcode_011_o_o_g_m_after2("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_011_OOGMAfter2.json");
        fn static_call_sha256_4("tests/GeneralStateTests/stStaticCall/static_CallSha256_4.json");
        fn static_call_identitiy_1("tests/GeneralStateTests/stStaticCall/static_CallIdentitiy_1.json");
        fn static_callcallcode_01_2("tests/GeneralStateTests/stStaticCall/static_callcallcode_01_2.json");
        fn static_log1_max_topic("tests/GeneralStateTests/stStaticCall/static_log1_MaxTopic.json");
        fn static_call_recursive_bomb0("tests/GeneralStateTests/stStaticCall/static_CallRecursiveBomb0.json");
        fn static_call_ecrecover0_overlapping_input_output("tests/GeneralStateTests/stStaticCall/static_CallEcrecover0_overlappingInputOutput.json");
        fn static_log1_log_memsize_too_high("tests/GeneralStateTests/stStaticCall/static_log1_logMemsizeTooHigh.json");
        fn static_callcallcodecallcode_011_o_o_g_m_after("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_011_OOGMAfter.json");
        fn static_call_ecrecover0_gas3000("tests/GeneralStateTests/stStaticCall/static_CallEcrecover0_gas3000.json");
        fn static_callcodecallcallcode_101_o_o_g_m_after_1("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_101_OOGMAfter_1.json");
        fn static_a_b_acalls3("tests/GeneralStateTests/stStaticCall/static_ABAcalls3.json");
        fn static_callcodecallcodecall_1102("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_1102.json");
        fn static_call_sha256_4_gas99("tests/GeneralStateTests/stStaticCall/static_CallSha256_4_gas99.json");
        fn static_call_output1("tests/GeneralStateTests/stStaticCall/static_callOutput1.json");
        fn static_call_ecrecover_h_prefixed0("tests/GeneralStateTests/stStaticCall/static_CallEcrecoverH_prefixed0.json");
        fn static_callcallcodecallcode_011_o_o_g_m_after_1("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_011_OOGMAfter_1.json");
        fn static_contract_creation_make_call_that_ask_more_gas_then_transaction_provided("tests/GeneralStateTests/stStaticCall/static_contractCreationMakeCallThatAskMoreGasThenTransactionProvided.json");
        fn static_callcallcall_000_suicide_middle("tests/GeneralStateTests/stStaticCall/static_callcallcall_000_SuicideMiddle.json");
        fn static_call_ripemd160_3_prefixed0("tests/GeneralStateTests/stStaticCall/static_CallRipemd160_3_prefixed0.json");
        fn static_call50000bytes_contract50_3("tests/GeneralStateTests/stStaticCall/static_Call50000bytesContract50_3.json");
        fn static_call_ripemd160_5("tests/GeneralStateTests/stStaticCall/static_CallRipemd160_5.json");
        fn static_callcallcallcode_001_o_o_g_m_after("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_001_OOGMAfter.json");
        fn static_execute_call_that_ask_fore_gas_then_trabsaction_has("tests/GeneralStateTests/stStaticCall/static_ExecuteCallThatAskForeGasThenTrabsactionHas.json");
        fn static_log0_non_empty_mem_log_mem_size1("tests/GeneralStateTests/stStaticCall/static_log0_nonEmptyMem_logMemSize1.json");
        fn static_a_b_acalls_suicide1("tests/GeneralStateTests/stStaticCall/static_ABAcallsSuicide1.json");
        fn staticcall_to_precompile_from_contract_initialization("tests/GeneralStateTests/stStaticCall/StaticcallToPrecompileFromContractInitialization.json");
        fn static_callcodecall_10("tests/GeneralStateTests/stStaticCall/static_callcodecall_10.json");
        fn static_a_b_acalls_suicide0("tests/GeneralStateTests/stStaticCall/static_ABAcallsSuicide0.json");
        fn static_refund_call_to_suicide_twice("tests/GeneralStateTests/stStaticCall/static_refund_CallToSuicideTwice.json");
        fn static_c_r_e_a_t_e_empty_contract_with_storage_and_call_it_0wei("tests/GeneralStateTests/stStaticCall/static_CREATE_EmptyContractWithStorageAndCallIt_0wei.json");
        fn static_internal_call_hitting_gas_limit2("tests/GeneralStateTests/stStaticCall/static_InternalCallHittingGasLimit2.json");
        fn static_call_contract_to_create_contract_and_call_it_o_o_g("tests/GeneralStateTests/stStaticCall/static_CallContractToCreateContractAndCallItOOG.json");
        fn static_call_recursive_bomb0_o_o_g_at_max_call_depth("tests/GeneralStateTests/stStaticCall/static_CallRecursiveBomb0_OOG_atMaxCallDepth.json");
        fn static_callcallcode_01_suicide_end2("tests/GeneralStateTests/stStaticCall/static_callcallcode_01_SuicideEnd2.json");
        fn static_call50000bytes_contract50_2("tests/GeneralStateTests/stStaticCall/static_Call50000bytesContract50_2.json");
        fn static_call_ripemd160_4("tests/GeneralStateTests/stStaticCall/static_CallRipemd160_4.json");
        fn static_callcallcallcode_001_2("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_001_2.json");
        fn static_callcallcodecallcode_011("tests/GeneralStateTests/stStaticCall/static_callcallcodecallcode_011.json");
        fn static_call1024_o_o_g("tests/GeneralStateTests/stStaticCall/static_Call1024OOG.json");
        fn static_call_sha256_3_postfix0("tests/GeneralStateTests/stStaticCall/static_CallSha256_3_postfix0.json");
        fn static_a_b_acalls2("tests/GeneralStateTests/stStaticCall/static_ABAcalls2.json");
        fn static_call_contract_to_create_contract_which_would_create_contract_if_called("tests/GeneralStateTests/stStaticCall/static_CallContractToCreateContractWhichWouldCreateContractIfCalled.json");
        fn static_call_o_o_g_additional_gas_costs2("tests/GeneralStateTests/stStaticCall/static_call_OOG_additionalGasCosts2.json");
        fn static_call_change_revert("tests/GeneralStateTests/stStaticCall/static_callChangeRevert.json");
        fn static_callcallcodecall_010_o_o_g_m_before("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_010_OOGMBefore.json");
        fn static_call_recursive_bomb1("tests/GeneralStateTests/stStaticCall/static_CallRecursiveBomb1.json");
        fn static_callcodecallcall_a_b_c_b_r_e_c_u_r_s_i_v_e("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_ABCB_RECURSIVE.json");
        fn static_callcodecallcallcode_101_o_o_g_m_after2("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_101_OOGMAfter2.json");
        fn static_call_sha256_5("tests/GeneralStateTests/stStaticCall/static_CallSha256_5.json");
        fn static_callcallcodecall_010_suicide_middle2("tests/GeneralStateTests/stStaticCall/static_callcallcodecall_010_SuicideMiddle2.json");
        fn static_callcodecallcall_100_o_o_g_m_after2("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100_OOGMAfter2.json");
        fn static_callcallcallcode_001("tests/GeneralStateTests/stStaticCall/static_callcallcallcode_001.json");
        fn static_callcodecallcodecall_110_suicide_middle2("tests/GeneralStateTests/stStaticCall/static_callcodecallcodecall_110_SuicideMiddle2.json");
        fn static_call_to_call_op_code_check("tests/GeneralStateTests/stStaticCall/static_callToCallOpCodeCheck.json");
        fn static_loop_calls_depth_then_revert("tests/GeneralStateTests/stStaticCall/static_LoopCallsDepthThenRevert.json");
        fn static_callcodecallcallcode_101_suicide_middle("tests/GeneralStateTests/stStaticCall/static_callcodecallcallcode_101_SuicideMiddle.json");
    }
}

mod st_mem_expanding_e_i_p150_calls {
    define_tests! {

        // --- ALL PASS ---
        fn delegate_call_on_e_i_p_with_mem_expanding_calls("tests/GeneralStateTests/stMemExpandingEIP150Calls/DelegateCallOnEIPWithMemExpandingCalls.json");
        fn call_and_callcode_consume_more_gas_then_transaction_has_with_mem_expanding_calls("tests/GeneralStateTests/stMemExpandingEIP150Calls/CallAndCallcodeConsumeMoreGasThenTransactionHasWithMemExpandingCalls.json");
        fn new_gas_price_for_codes_with_mem_expanding_calls("tests/GeneralStateTests/stMemExpandingEIP150Calls/NewGasPriceForCodesWithMemExpandingCalls.json");
        fn o_o_gin_return("tests/GeneralStateTests/stMemExpandingEIP150Calls/OOGinReturn.json");
        fn create_and_gas_inside_create_with_mem_expanding_calls("tests/GeneralStateTests/stMemExpandingEIP150Calls/CreateAndGasInsideCreateWithMemExpandingCalls.json");
        fn call_ask_more_gas_on_depth2_then_transaction_has_with_mem_expanding_calls("tests/GeneralStateTests/stMemExpandingEIP150Calls/CallAskMoreGasOnDepth2ThenTransactionHasWithMemExpandingCalls.json");
        fn call_goes_o_o_g_on_second_level2_with_mem_expanding_calls("tests/GeneralStateTests/stMemExpandingEIP150Calls/CallGoesOOGOnSecondLevel2WithMemExpandingCalls.json");
        fn call_goes_o_o_g_on_second_level_with_mem_expanding_calls("tests/GeneralStateTests/stMemExpandingEIP150Calls/CallGoesOOGOnSecondLevelWithMemExpandingCalls.json");
        fn execute_call_that_ask_more_gas_then_transaction_has_with_mem_expanding_calls("tests/GeneralStateTests/stMemExpandingEIP150Calls/ExecuteCallThatAskMoreGasThenTransactionHasWithMemExpandingCalls.json");
    }
}

mod st_args_zero_one_balance {
    define_tests! {

        // --- ALL PASS ---
        fn addmod_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/addmodNonConst.json");
        fn add_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/addNonConst.json");
        fn not_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/notNonConst.json");
        fn jumpi_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/jumpiNonConst.json");
        fn extcodesize_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/extcodesizeNonConst.json");
        fn return_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/returnNonConst.json");
        fn sload_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/sloadNonConst.json");
        fn sstore_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/sstoreNonConst.json");
        fn mstore_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/mstoreNonConst.json");
        fn codecopy_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/codecopyNonConst.json");
        fn mstore8_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/mstore8NonConst.json");
        fn div_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/divNonConst.json");
        fn exp_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/expNonConst.json");
        fn mload_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/mloadNonConst.json");
        fn lt_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/ltNonConst.json");
        fn log1_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/log1NonConst.json");
        fn or_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/orNonConst.json");
        fn iszero_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/iszeroNonConst.json");
        fn extcodecopy_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/extcodecopyNonConst.json");
        fn sgt_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/sgtNonConst.json");
        fn sdiv_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/sdivNonConst.json");
        fn and_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/andNonConst.json");
        fn calldatacopy_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/calldatacopyNonConst.json");
        fn log0_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/log0NonConst.json");
        fn callcode_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/callcodeNonConst.json");
        fn jump_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/jumpNonConst.json");
        fn calldataload_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/calldataloadNonConst.json");
        fn signext_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/signextNonConst.json");
        fn sha3_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/sha3NonConst.json");
        fn log2_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/log2NonConst.json");
        fn xor_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/xorNonConst.json");
        fn balance_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/balanceNonConst.json");
        fn create_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/createNonConst.json");
        fn mulmod_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/mulmodNonConst.json");
        fn log3_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/log3NonConst.json");
        fn mod_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/modNonConst.json");
        fn gt_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/gtNonConst.json");
        fn byte_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/byteNonConst.json");
        fn suicide_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/suicideNonConst.json");
        fn mul_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/mulNonConst.json");
        fn delegatecall_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/delegatecallNonConst.json");
        fn eq_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/eqNonConst.json");
        fn sub_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/subNonConst.json");
        fn call_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/callNonConst.json");
        fn slt_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/sltNonConst.json");
        fn smod_non_const("tests/GeneralStateTests/stArgsZeroOneBalance/smodNonConst.json");
    }
}

mod v_m_tests {
    define_tests! {

        // --- MOST PASS --- (1 test fails with gas issue)
        fn vm_arithmetic_test_mulmod("tests/GeneralStateTests/VMTests/vmArithmeticTest/mulmod.json");
        fn vm_arithmetic_test_mod("tests/GeneralStateTests/VMTests/vmArithmeticTest/mod.json");
        fn vm_arithmetic_test_not("tests/GeneralStateTests/VMTests/vmArithmeticTest/not.json");
        fn vm_arithmetic_test_smod("tests/GeneralStateTests/VMTests/vmArithmeticTest/smod.json");
        fn vm_arithmetic_test_sub("tests/GeneralStateTests/VMTests/vmArithmeticTest/sub.json");
        fn vm_arithmetic_test_signextend("tests/GeneralStateTests/VMTests/vmArithmeticTest/signextend.json");
        fn vm_arithmetic_test_exp_power256_of256("tests/GeneralStateTests/VMTests/vmArithmeticTest/expPower256Of256.json");
        fn vm_arithmetic_test_two_ops("tests/GeneralStateTests/VMTests/vmArithmeticTest/twoOps.json");
        fn vm_arithmetic_test_arith("tests/GeneralStateTests/VMTests/vmArithmeticTest/arith.json");
        fn vm_arithmetic_test_addmod("tests/GeneralStateTests/VMTests/vmArithmeticTest/addmod.json");
        fn vm_arithmetic_test_fib("tests/GeneralStateTests/VMTests/vmArithmeticTest/fib.json");
        fn vm_arithmetic_test_div("tests/GeneralStateTests/VMTests/vmArithmeticTest/div.json");
        fn vm_arithmetic_test_div_by_zero("tests/GeneralStateTests/VMTests/vmArithmeticTest/divByZero.json");
        fn vm_arithmetic_test_exp("tests/GeneralStateTests/VMTests/vmArithmeticTest/exp.json");
        fn vm_arithmetic_test_add("tests/GeneralStateTests/VMTests/vmArithmeticTest/add.json");
        fn vm_arithmetic_test_exp_power256("tests/GeneralStateTests/VMTests/vmArithmeticTest/expPower256.json");
        fn vm_arithmetic_test_mul("tests/GeneralStateTests/VMTests/vmArithmeticTest/mul.json");
        fn vm_arithmetic_test_sdiv("tests/GeneralStateTests/VMTests/vmArithmeticTest/sdiv.json");
        fn vm_arithmetic_test_exp_power2("tests/GeneralStateTests/VMTests/vmArithmeticTest/expPower2.json");

        fn vm_bitwise_logic_operation_sgt("tests/GeneralStateTests/VMTests/vmBitwiseLogicOperation/sgt.json");
        fn vm_bitwise_logic_operation_or("tests/GeneralStateTests/VMTests/vmBitwiseLogicOperation/or.json");
        fn vm_bitwise_logic_operation_lt("tests/GeneralStateTests/VMTests/vmBitwiseLogicOperation/lt.json");
        fn vm_bitwise_logic_operation_eq("tests/GeneralStateTests/VMTests/vmBitwiseLogicOperation/eq.json");
        fn vm_bitwise_logic_operation_not("tests/GeneralStateTests/VMTests/vmBitwiseLogicOperation/not.json");
        fn vm_bitwise_logic_operation_xor("tests/GeneralStateTests/VMTests/vmBitwiseLogicOperation/xor.json");
        fn vm_bitwise_logic_operation_and("tests/GeneralStateTests/VMTests/vmBitwiseLogicOperation/and.json");
        fn vm_bitwise_logic_operation_slt("tests/GeneralStateTests/VMTests/vmBitwiseLogicOperation/slt.json");
        fn vm_bitwise_logic_operation_gt("tests/GeneralStateTests/VMTests/vmBitwiseLogicOperation/gt.json");
        fn vm_bitwise_logic_operation_iszero("tests/GeneralStateTests/VMTests/vmBitwiseLogicOperation/iszero.json");
        fn vm_bitwise_logic_operation_byte("tests/GeneralStateTests/VMTests/vmBitwiseLogicOperation/byte.json");

        fn vm_i_oand_flow_operations_msize("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/msize.json");
        fn vm_i_oand_flow_operations_mload("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/mload.json");
        fn vm_i_oand_flow_operations_sstore_sload("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/sstore_sload.json");
        fn vm_i_oand_flow_operations_return("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/return.json");
        fn vm_i_oand_flow_operations_gas("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/gas.json");
        fn vm_i_oand_flow_operations_codecopy("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/codecopy.json");
        fn vm_i_oand_flow_operations_jumpi("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/jumpi.json");
        fn vm_i_oand_flow_operations_loop_stacklimit("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/loop_stacklimit.json");
        fn vm_i_oand_flow_operations_jump("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/jump.json");
        fn vm_i_oand_flow_operations_mstore("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/mstore.json");
        fn vm_i_oand_flow_operations_loops_conditionals("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/loopsConditionals.json");
        fn vm_i_oand_flow_operations_pc("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/pc.json");
        fn vm_i_oand_flow_operations_mstore8("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/mstore8.json");
        fn vm_i_oand_flow_operations_jump_to_push("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/jumpToPush.json");
        fn vm_i_oand_flow_operations_pop("tests/GeneralStateTests/VMTests/vmIOandFlowOperations/pop.json");

        fn vm_log_test_log0("tests/GeneralStateTests/VMTests/vmLogTest/log0.json");
        fn vm_log_test_log1("tests/GeneralStateTests/VMTests/vmLogTest/log1.json");
        fn vm_log_test_log4("tests/GeneralStateTests/VMTests/vmLogTest/log4.json");
        fn vm_log_test_log2("tests/GeneralStateTests/VMTests/vmLogTest/log2.json");
        fn vm_log_test_log3("tests/GeneralStateTests/VMTests/vmLogTest/log3.json");

        fn vm_performance_performance_tester("tests/GeneralStateTests/VMTests/vmPerformance/performanceTester.json");
        fn vm_performance_loop_exp("tests/GeneralStateTests/VMTests/vmPerformance/loopExp.json");
        fn vm_performance_loop_mul("tests/GeneralStateTests/VMTests/vmPerformance/loopMul.json");

        fn vm_tests_block_info("tests/GeneralStateTests/VMTests/vmTests/blockInfo.json");
        fn vm_tests_suicide("tests/GeneralStateTests/VMTests/vmTests/suicide.json");
        fn vm_tests_env_info("tests/GeneralStateTests/VMTests/vmTests/envInfo.json");
        fn vm_tests_sha3("tests/GeneralStateTests/VMTests/vmTests/sha3.json");
        fn vm_tests_calldatacopy("tests/GeneralStateTests/VMTests/vmTests/calldatacopy.json");
        fn vm_tests_random("tests/GeneralStateTests/VMTests/vmTests/random.json");
        fn vm_tests_push("tests/GeneralStateTests/VMTests/vmTests/push.json");
        fn vm_tests_calldatasize("tests/GeneralStateTests/VMTests/vmTests/calldatasize.json");
        fn vm_tests_dup("tests/GeneralStateTests/VMTests/vmTests/dup.json");
        fn vm_tests_swap("tests/GeneralStateTests/VMTests/vmTests/swap.json");
        fn vm_tests_calldataload("tests/GeneralStateTests/VMTests/vmTests/calldataload.json");
    }
}

mod st_attack_test {
    define_tests! {

        // -- ALL PASS ---
        fn st_attack_test_crashing_transaction("tests/GeneralStateTests/stAttackTest/CrashingTransaction.json");
        fn st_attack_test_contract_creation_spam("tests/GeneralStateTests/stAttackTest/ContractCreationSpam.json");
    }
}

mod pyspecs {
    define_tests! {

        // -- ALL PASS ---
        fn homestead_yul_yul("tests/GeneralStateTests/Pyspecs/homestead/yul/yul.json");

        // -- MOST PASS --- (2 code errors)
        fn shanghai_eip3860_initcode_create_opcode_initcode("tests/GeneralStateTests/Pyspecs/shanghai/eip3860_initcode/create_opcode_initcode.json");
        fn shanghai_eip3860_initcode_gas_usage("tests/GeneralStateTests/Pyspecs/shanghai/eip3860_initcode/gas_usage.json");
        fn shanghai_eip3860_initcode_contract_creating_tx("tests/GeneralStateTests/Pyspecs/shanghai/eip3860_initcode/contract_creating_tx.json");
        fn shanghai_eip3855_push0_push0_fill_stack("tests/GeneralStateTests/Pyspecs/shanghai/eip3855_push0/push0_fill_stack.json");
        fn shanghai_eip3855_push0_push0_stack_overflow("tests/GeneralStateTests/Pyspecs/shanghai/eip3855_push0/push0_stack_overflow.json");
        fn shanghai_eip3855_push0_push0_key_sstore("tests/GeneralStateTests/Pyspecs/shanghai/eip3855_push0/push0_key_sstore.json");
        fn shanghai_eip3855_push0_push0_gas_cost("tests/GeneralStateTests/Pyspecs/shanghai/eip3855_push0/push0_gas_cost.json");
        fn shanghai_eip3855_push0_push0_storage_overwrite("tests/GeneralStateTests/Pyspecs/shanghai/eip3855_push0/push0_storage_overwrite.json");
        fn shanghai_eip3855_push0_push0_before_jumpdest("tests/GeneralStateTests/Pyspecs/shanghai/eip3855_push0/push0_before_jumpdest.json");
        fn shanghai_eip3855_push0_push0_during_staticcall("tests/GeneralStateTests/Pyspecs/shanghai/eip3855_push0/push0_during_staticcall.json");
        fn shanghai_eip3651_warm_coinbase_warm_coinbase_call_out_of_gas("tests/GeneralStateTests/Pyspecs/shanghai/eip3651_warm_coinbase/warm_coinbase_call_out_of_gas.json");
        fn shanghai_eip3651_warm_coinbase_warm_coinbase_gas_usage("tests/GeneralStateTests/Pyspecs/shanghai/eip3651_warm_coinbase/warm_coinbase_gas_usage.json");

        // -- ALL PASS ---
        fn cancun_eip5656_mcopy_mcopy_memory_expansion("tests/GeneralStateTests/Pyspecs/cancun/eip5656_mcopy/mcopy_memory_expansion.json");
        fn cancun_eip5656_mcopy_mcopy_on_empty_memory("tests/GeneralStateTests/Pyspecs/cancun/eip5656_mcopy/mcopy_on_empty_memory.json");
        fn cancun_eip5656_mcopy_mcopy_huge_memory_expansion("tests/GeneralStateTests/Pyspecs/cancun/eip5656_mcopy/mcopy_huge_memory_expansion.json");
        fn cancun_eip5656_mcopy_valid_mcopy_operations("tests/GeneralStateTests/Pyspecs/cancun/eip5656_mcopy/valid_mcopy_operations.json");
        fn cancun_eip5656_mcopy_no_memory_corruption_on_upper_call_stack_levels("tests/GeneralStateTests/Pyspecs/cancun/eip5656_mcopy/no_memory_corruption_on_upper_call_stack_levels.json");
        fn cancun_eip1153_tstore_subcall("tests/GeneralStateTests/Pyspecs/cancun/eip1153_tstore/subcall.json");
        fn cancun_eip1153_tstore_gas_usage("tests/GeneralStateTests/Pyspecs/cancun/eip1153_tstore/gas_usage.json");
        fn cancun_eip1153_tstore_transient_storage_unset_values("tests/GeneralStateTests/Pyspecs/cancun/eip1153_tstore/transient_storage_unset_values.json");
        fn cancun_eip1153_tstore_run_until_out_of_gas("tests/GeneralStateTests/Pyspecs/cancun/eip1153_tstore/run_until_out_of_gas.json");
        fn cancun_eip1153_tstore_contract_creation("tests/GeneralStateTests/Pyspecs/cancun/eip1153_tstore/contract_creation.json");
        fn cancun_eip1153_tstore_tload_after_tstore_is_zero("tests/GeneralStateTests/Pyspecs/cancun/eip1153_tstore/tload_after_tstore_is_zero.json");
        fn cancun_eip1153_tstore_tload_after_tstore("tests/GeneralStateTests/Pyspecs/cancun/eip1153_tstore/tload_after_tstore.json");
        fn cancun_eip1153_tstore_reentrant_call("tests/GeneralStateTests/Pyspecs/cancun/eip1153_tstore/reentrant_call.json");
        fn cancun_eip1153_tstore_tload_after_sstore("tests/GeneralStateTests/Pyspecs/cancun/eip1153_tstore/tload_after_sstore.json");
        fn cancun_eip1153_tstore_reentrant_selfdestructing_call("tests/GeneralStateTests/Pyspecs/cancun/eip1153_tstore/reentrant_selfdestructing_call.json");
        fn cancun_eip4844_blobs_point_evaluation_precompile_gas_usage("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/point_evaluation_precompile_gas_usage.json");
        fn cancun_eip4844_blobs_point_evaluation_precompile_before_fork("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/point_evaluation_precompile_before_fork.json");
        fn cancun_eip4844_blobs_blob_gas_subtraction_tx("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blob_gas_subtraction_tx.json");
        fn cancun_eip4844_blobs_insufficient_balance_blob_tx("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/insufficient_balance_blob_tx.json");
        fn cancun_eip4844_blobs_sufficient_balance_blob_tx("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/sufficient_balance_blob_tx.json");
        fn cancun_eip4844_blobs_point_evaluation_precompile_calls("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/point_evaluation_precompile_calls.json");
        fn cancun_eip4844_blobs_invalid_blob_hash_versioning_single_tx("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/invalid_blob_hash_versioning_single_tx.json");
        fn cancun_eip4844_blobs_blob_type_tx_pre_fork("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blob_type_tx_pre_fork.json");
        fn cancun_eip4844_blobs_invalid_normal_gas("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/invalid_normal_gas.json");
        fn cancun_eip4844_blobs_invalid_tx_max_fee_per_blob_gas_state("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/invalid_tx_max_fee_per_blob_gas_state.json");
        fn cancun_eip4844_blobs_valid_precompile_calls("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/valid_precompile_calls.json");
        fn cancun_eip4844_blobs_point_evaluation_precompile_external_vectors("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/point_evaluation_precompile_external_vectors.json");
        fn cancun_eip4844_blobs_blob_tx_attribute_gasprice_opcode("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blob_tx_attribute_gasprice_opcode.json");
        fn cancun_eip4844_blobs_blob_tx_attribute_value_opcode("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blob_tx_attribute_value_opcode.json");
        fn cancun_eip4844_blobs_blob_tx_attribute_calldata_opcodes("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blob_tx_attribute_calldata_opcodes.json");
        fn cancun_eip4844_blobs_point_evaluation_precompile_gas_tx_to("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/point_evaluation_precompile_gas_tx_to.json");
        fn cancun_eip4844_blobs_invalid_tx_blob_count("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/invalid_tx_blob_count.json");
        fn cancun_eip4844_blobs_invalid_precompile_calls("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/invalid_precompile_calls.json");
        fn cancun_eip4844_blobs_blob_tx_attribute_opcodes("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blob_tx_attribute_opcodes.json");
        fn cancun_eip6780_selfdestruct_delegatecall_from_pre_existing_contract_to_new_contract("tests/GeneralStateTests/Pyspecs/cancun/eip6780_selfdestruct/delegatecall_from_pre_existing_contract_to_new_contract.json");
        fn cancun_eip6780_selfdestruct_create_selfdestruct_same_tx("tests/GeneralStateTests/Pyspecs/cancun/eip6780_selfdestruct/create_selfdestruct_same_tx.json");
        fn cancun_eip6780_selfdestruct_selfdestruct_not_created_in_same_tx_with_revert("tests/GeneralStateTests/Pyspecs/cancun/eip6780_selfdestruct/selfdestruct_not_created_in_same_tx_with_revert.json");
        fn cancun_eip6780_selfdestruct_selfdestruct_created_in_same_tx_with_revert("tests/GeneralStateTests/Pyspecs/cancun/eip6780_selfdestruct/selfdestruct_created_in_same_tx_with_revert.json");
        fn cancun_eip6780_selfdestruct_reentrancy_selfdestruct_revert("tests/GeneralStateTests/Pyspecs/cancun/eip6780_selfdestruct/reentrancy_selfdestruct_revert.json");
        fn cancun_eip6780_selfdestruct_self_destructing_initcode("tests/GeneralStateTests/Pyspecs/cancun/eip6780_selfdestruct/self_destructing_initcode.json");
        fn cancun_eip6780_selfdestruct_self_destructing_initcode_create_tx("tests/GeneralStateTests/Pyspecs/cancun/eip6780_selfdestruct/self_destructing_initcode_create_tx.json");
        fn cancun_eip6780_selfdestruct_delegatecall_from_new_contract_to_pre_existing_contract("tests/GeneralStateTests/Pyspecs/cancun/eip6780_selfdestruct/delegatecall_from_new_contract_to_pre_existing_contract.json");
        fn cancun_eip6780_selfdestruct_dynamic_create2_selfdestruct_collision("tests/GeneralStateTests/Pyspecs/cancun/eip6780_selfdestruct/dynamic_create2_selfdestruct_collision.json");
        fn cancun_eip6780_selfdestruct_selfdestruct_pre_existing("tests/GeneralStateTests/Pyspecs/cancun/eip6780_selfdestruct/selfdestruct_pre_existing.json");
        fn cancun_eip7516_blobgasfee_blobbasefee_before_fork("tests/GeneralStateTests/Pyspecs/cancun/eip7516_blobgasfee/blobbasefee_before_fork.json");
        fn cancun_eip7516_blobgasfee_blobbasefee_out_of_gas("tests/GeneralStateTests/Pyspecs/cancun/eip7516_blobgasfee/blobbasefee_out_of_gas.json");
        fn cancun_eip7516_blobgasfee_blobbasefee_stack_overflow("tests/GeneralStateTests/Pyspecs/cancun/eip7516_blobgasfee/blobbasefee_stack_overflow.json");
        fn istanbul_eip1344_chainid_chainid("tests/GeneralStateTests/Pyspecs/istanbul/eip1344_chainid/chainid.json");
        fn frontier_opcodes_value_transfer_gas_calculation("tests/GeneralStateTests/Pyspecs/frontier/opcodes/value_transfer_gas_calculation.json");
        fn frontier_opcodes_dup("tests/GeneralStateTests/Pyspecs/frontier/opcodes/dup.json");
        fn berlin_eip2930_access_list_access_list("tests/GeneralStateTests/Pyspecs/berlin/eip2930_access_list/access_list.json");
        fn byzantium_eip198_modexp_precompile_modexp("tests/GeneralStateTests/Pyspecs/byzantium/eip198_modexp_precompile/modexp.json");
    }
}

mod st_return_data_test {
    define_tests! {

        // --- ALL PASS ---
        fn returndatacopy_following_too_big_transfer("tests/GeneralStateTests/stReturnDataTest/returndatacopy_following_too_big_transfer.json");
        fn returndatasize_bug("tests/GeneralStateTests/stReturnDataTest/returndatasize_bug.json");
        fn returndatasize_initial_zero_read("tests/GeneralStateTests/stReturnDataTest/returndatasize_initial_zero_read.json");
        fn returndatasize_following_successful_create("tests/GeneralStateTests/stReturnDataTest/returndatasize_following_successful_create.json");
        fn returndatacopy_following_revert_in_create("tests/GeneralStateTests/stReturnDataTest/returndatacopy_following_revert_in_create.json");
        fn returndatacopy_following_failing_call("tests/GeneralStateTests/stReturnDataTest/returndatacopy_following_failing_call.json");
        fn returndatacopy_following_revert("tests/GeneralStateTests/stReturnDataTest/returndatacopy_following_revert.json");
        fn subcall_return_more_then_expected("tests/GeneralStateTests/stReturnDataTest/subcallReturnMoreThenExpected.json");
        fn returndatacopy_after_failing_callcode("tests/GeneralStateTests/stReturnDataTest/returndatacopy_after_failing_callcode.json");
        fn returndatacopy_after_failing_create("tests/GeneralStateTests/stReturnDataTest/returndatacopy_afterFailing_create.json");
        fn returndatacopy_following_successful_create("tests/GeneralStateTests/stReturnDataTest/returndatacopy_following_successful_create.json");
        fn returndatacopy_after_failing_staticcall("tests/GeneralStateTests/stReturnDataTest/returndatacopy_after_failing_staticcall.json");
        fn returndatasize_after_failing_delegatecall("tests/GeneralStateTests/stReturnDataTest/returndatasize_after_failing_delegatecall.json");
        fn create_callprecompile_returndatasize("tests/GeneralStateTests/stReturnDataTest/create_callprecompile_returndatasize.json");
        fn returndatacopy_overrun("tests/GeneralStateTests/stReturnDataTest/returndatacopy_overrun.json");
        fn call_then_call_value_fail_then_returndatasize("tests/GeneralStateTests/stReturnDataTest/call_then_call_value_fail_then_returndatasize.json");
        fn returndatasize_after_failing_staticcall("tests/GeneralStateTests/stReturnDataTest/returndatasize_after_failing_staticcall.json");
        fn call_then_create_successful_then_returndatasize("tests/GeneralStateTests/stReturnDataTest/call_then_create_successful_then_returndatasize.json");
        fn returndatasize_after_successful_delegatecall("tests/GeneralStateTests/stReturnDataTest/returndatasize_after_successful_delegatecall.json");
        fn returndatacopy_following_create("tests/GeneralStateTests/stReturnDataTest/returndatacopy_following_create.json");
        fn revert_ret_data_size("tests/GeneralStateTests/stReturnDataTest/revertRetDataSize.json");
        fn too_long_return_data_copy("tests/GeneralStateTests/stReturnDataTest/tooLongReturnDataCopy.json");
        fn returndatacopy_initial_big_sum("tests/GeneralStateTests/stReturnDataTest/returndatacopy_initial_big_sum.json");
        fn call_ecrec_success_empty_then_returndatasize("tests/GeneralStateTests/stReturnDataTest/call_ecrec_success_empty_then_returndatasize.json");
        fn returndatacopy_initial("tests/GeneralStateTests/stReturnDataTest/returndatacopy_initial.json");
        fn returndatacopy_following_call("tests/GeneralStateTests/stReturnDataTest/returndatacopy_following_call.json");
        fn returndatacopy_after_successful_callcode("tests/GeneralStateTests/stReturnDataTest/returndatacopy_after_successful_callcode.json");
        fn returndatasize_after_oog_after_deeper("tests/GeneralStateTests/stReturnDataTest/returndatasize_after_oog_after_deeper.json");
        fn modexp_modsize0_returndatasize("tests/GeneralStateTests/stReturnDataTest/modexp_modsize0_returndatasize.json");
        fn returndatacopy_initial_256("tests/GeneralStateTests/stReturnDataTest/returndatacopy_initial_256.json");
        fn returndatacopy_after_successful_staticcall("tests/GeneralStateTests/stReturnDataTest/returndatacopy_after_successful_staticcall.json");
        fn returndatacopy_0_0_following_successful_create("tests/GeneralStateTests/stReturnDataTest/returndatacopy_0_0_following_successful_create.json");
        fn clear_return_buffer("tests/GeneralStateTests/stReturnDataTest/clearReturnBuffer.json");
        fn call_outsize_then_create_successful_then_returndatasize("tests/GeneralStateTests/stReturnDataTest/call_outsize_then_create_successful_then_returndatasize.json");
        fn returndatacopy_after_revert_in_staticcall("tests/GeneralStateTests/stReturnDataTest/returndatacopy_after_revert_in_staticcall.json");
        fn returndatasize_after_successful_callcode("tests/GeneralStateTests/stReturnDataTest/returndatasize_after_successful_callcode.json");
        fn returndatacopy_after_failing_delegatecall("tests/GeneralStateTests/stReturnDataTest/returndatacopy_after_failing_delegatecall.json");
        fn returndatasize_after_successful_staticcall("tests/GeneralStateTests/stReturnDataTest/returndatasize_after_successful_staticcall.json");
        fn returndatasize_initial("tests/GeneralStateTests/stReturnDataTest/returndatasize_initial.json");
        fn returndatacopy_after_successful_delegatecall("tests/GeneralStateTests/stReturnDataTest/returndatacopy_after_successful_delegatecall.json");
        fn returndatasize_after_failing_callcode("tests/GeneralStateTests/stReturnDataTest/returndatasize_after_failing_callcode.json");
    }
}
