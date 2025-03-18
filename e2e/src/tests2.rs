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
        fn st_e_i_p3651_warmcoinbase_coinbase_warm_account_call_gas_fail("tests/GeneralStateTests/Shanghai/stEIP3651-warmcoinbase/coinbaseWarmAccountCallGasFail.json");
        // fn st_e_i_p3651_warmcoinbase_coinbase_warm_account_call_gas("tests/GeneralStateTests/Shanghai/stEIP3651-warmcoinbase/coinbaseWarmAccountCallGas.json");
    }
}
