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
        // fn random_statetest649("tests/GeneralStateTests/stRandom2/randomStatetest649.json");
        // fn static_callcall_00_o_o_g_e("tests/GeneralStateTests/stStaticCall/static_callcall_00_OOGE.json");
        fn static_callcodecallcall_100_o_o_g_e("tests/GeneralStateTests/stStaticCall/static_callcodecallcall_100_OOGE.json");
    }
}
