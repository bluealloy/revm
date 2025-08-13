macro_rules! define_tests {
    (
        $( fn $test_name:ident($test_path:literal); )*
    ) => {
        $(
            #[test]
            fn $test_name() {
                $crate::utils::run_e2e_test($test_path)
            }
        )*
    };
}

mod good_coverage_tests {
    define_tests! {
        fn st_e_i_p3860_limitmeterinitcode_create_init_code_size_limit("tests/GeneralStateTests/Shanghai/stEIP3860-limitmeterinitcode/createInitCodeSizeLimit.json");
        fn st_e_i_p3651_warmcoinbase_coinbase_warm_account_call_gas_fail("tests/GeneralStateTests/Shanghai/stEIP3651-warmcoinbase/coinbaseWarmAccountCallGasFail.json");
        fn st_e_i_p3651_warmcoinbase_coinbase_warm_account_call_gas("tests/GeneralStateTests/Shanghai/stEIP3651-warmcoinbase/coinbaseWarmAccountCallGas.json");
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
        fn cancun_eip1153_tstore_contract_creation("tests/GeneralStateTests/Pyspecs/cancun/eip1153_tstore/contract_creation.json");
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

        // this test can't pass because it relays on a modified EVM precompiled contract,
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

        // infinite loop or a very long test
        // fn st_attack_test_contract_creation_spam("tests/GeneralStateTests/stAttackTest/ContractCreationSpam.json");

        fn high_gas_price_paris("tests/GeneralStateTests/stTransactionTest/HighGasPriceParis.json");

        // fn return50000("tests/GeneralStateTests/stQuadraticComplexityTest/Return50000.json");
        // fn return50000_2("tests/GeneralStateTests/stQuadraticComplexityTest/Return50000_2.json");
        // fn static_call50000("tests/GeneralStateTests/stStaticCall/static_Call50000.json");
        // fn static_call50000_ecrec("tests/GeneralStateTests/stStaticCall/static_Call50000_ecrec.json");
        // fn static_call50000_identity2("tests/GeneralStateTests/stStaticCall/static_Call50000_identity2.json");
        // fn static_loop_calls_depth_then_revert2("tests/GeneralStateTests/stStaticCall/static_LoopCallsDepthThenRevert2.json");
        // fn static_loop_calls_depth_then_revert3("tests/GeneralStateTests/stStaticCall/static_LoopCallsDepthThenRevert3.json");
        // fn static_return50000_2("tests/GeneralStateTests/stStaticCall/static_Return50000_2.json");
    }
}

mod stack_underflow_tests {
    define_tests! {
        // fn trans_storage_ok("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/transStorageOK.json");
        fn invalid_diff_places("tests/GeneralStateTests/stBadOpcode/invalidDiffPlaces.json");
    }
}

mod fails_with_stack_expansion {
    define_tests! {
        fn _15_tstore_cannot_be_dosd("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/15_tstoreCannotBeDosd.json");
        fn _21_tstore_cannot_be_dosd_ooo("tests/GeneralStateTests/Cancun/stEIP1153-transientStorage/21_tstoreCannotBeDosdOOO.json");
        fn contract_creation_spam("tests/GeneralStateTests/stAttackTest/ContractCreationSpam.json");
    }
}

mod test_test {
    define_tests! {
        fn code_copy_zero_paris("tests/GeneralStateTests/stExtCodeHash/codeCopyZero_Paris.json");
    }
}

mod cancun_st_eip4844_blobtransactions {
    define_tests! {
        // fn blobhash_list_bounds10("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds10.json");
        // fn blobhash_list_bounds3("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds3.json");
        // fn blobhash_list_bounds4("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds4.json");
        // fn blobhash_list_bounds5("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds5.json");
        // fn blobhash_list_bounds6("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds6.json");
        // fn blobhash_list_bounds7("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds7.json");
        // fn blobhash_list_bounds8("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds8.json");
        // fn blobhash_list_bounds9("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/blobhashListBounds9.json");
        // fn create_blobhash_tx("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/createBlobhashTx.json");
        // fn empty_blobhash_list("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/emptyBlobhashList.json");
        // fn opcode_blobh_bounds("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/opcodeBlobhBounds.json");
        // fn opcode_blobhash_out_of_range("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/opcodeBlobhashOutOfRange.json");
        // fn wrong_blobhash_version("tests/GeneralStateTests/Cancun/stEIP4844-blobtransactions/wrongBlobhashVersion.json");
    }
}

mod pyspecs_cancun_eip4844_blobs {
    define_tests! {
        // fn blob_gas_subtraction_tx("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blob_gas_subtraction_tx.json");
        // fn blob_tx_attribute_calldata_opcodes("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blob_tx_attribute_calldata_opcodes.json");
        // fn blob_tx_attribute_gasprice_opcode("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blob_tx_attribute_gasprice_opcode.json");
        // fn blob_tx_attribute_opcodes("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blob_tx_attribute_opcodes.json");
        // fn blob_tx_attribute_value_opcode("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blob_tx_attribute_value_opcode.json");
        // fn blob_type_tx_pre_fork("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blob_type_tx_pre_fork.json");
        // fn blobhash_gas_cost("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/blobhash_gas_cost.json");
        // fn call_opcode_types("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/call_opcode_types.json");
        // fn external_vectors("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/external_vectors.json");
        // fn insufficient_balance_blob_tx("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/insufficient_balance_blob_tx.json");
        // fn invalid_blob_hash_versioning_single_tx("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/invalid_blob_hash_versioning_single_tx.json");
        // fn invalid_inputs("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/invalid_inputs.json");
        // fn invalid_normal_gas("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/invalid_normal_gas.json");
        // fn invalid_tx_blob_count("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/invalid_tx_blob_count.json");
        // fn invalid_tx_max_fee_per_blob_gas_state("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/invalid_tx_max_fee_per_blob_gas_state.json");
        // fn point_evaluation_precompile_gas_usage("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/point_evaluation_precompile_gas_usage.json");
        // fn precompile_before_fork("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/precompile_before_fork.json");
        // fn sufficient_balance_blob_tx("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/sufficient_balance_blob_tx.json");
        // fn tx_entry_point("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/tx_entry_point.json");
        // fn valid_inputs("tests/GeneralStateTests/Pyspecs/cancun/eip4844_blobs/valid_inputs.json");
    }
}

mod state_root_mismatch {
    define_tests! {
        // fn create2collision_storage_paris("tests/GeneralStateTests/stCreate2/create2collisionStorageParis.json");
        // fn dynamic_account_overwrite_empty_paris("tests/GeneralStateTests/stExtCodeHash/dynamicAccountOverwriteEmpty_Paris.json");
        // fn init_collision_paris("tests/GeneralStateTests/stSStoreTest/InitCollisionParis.json");
        // fn revert_in_create_in_init_create2_paris("tests/GeneralStateTests/stCreate2/RevertInCreateInInitCreate2Paris.json");
        // fn revert_in_create_in_init_paris("tests/GeneralStateTests/stRevertTest/RevertInCreateInInit_Paris.json");
    }
}

mod v82_failing_tests {
    define_tests! {
        fn create2collision_selfdestructed("tests/GeneralStateTests/stCreate2/create2collisionSelfdestructed.json");
        fn create2collision_selfdestructed2("tests/GeneralStateTests/stCreate2/create2collisionSelfdestructed2.json");
        fn create2collision_selfdestructed_revert("tests/GeneralStateTests/stCreate2/create2collisionSelfdestructedRevert.json");
        fn create_acreate_b_bsuicide_bstore("tests/GeneralStateTests/stCreateTest/CREATE_AcreateB_BSuicide_BStore.json");
        fn failed_tx_xcf416c53_paris("tests/GeneralStateTests/stSpecialTest/failed_tx_xcf416c53_Paris.json");
        fn underflow_test("tests/GeneralStateTests/stStackTests/underflowTest.json");
    }
}
