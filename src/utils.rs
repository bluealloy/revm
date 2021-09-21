macro_rules! try_or_fail {
    ( $e:expr ) => {
        match $e {
            Ok(v) => v,
            Err(e) => return Capture::Exit((e.into(), None, Vec::new())),
        }
    };
}

pub fn l64(gas: u64) -> u64 {
    gas - gas / 64
}
