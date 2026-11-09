#![no_main]

use libfuzzer_sys::fuzz_target;
use tendermint::Time;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = core::str::from_utf8(data) else {
        return;
    };
    if let Ok(time) = s.parse::<Time>() {
        let reparsed: Time = time.to_rfc3339().parse().expect("re-parse failed");
        assert_eq!(time, reparsed);
    }
});
