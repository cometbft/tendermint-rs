#![no_main]

use libfuzzer_sys::fuzz_target;
use tendermint_p2p::secret_connection::Version;

fuzz_target!(|data: &[u8]| {
    for version in [Version::V0_34, Version::V0_33, Version::Legacy] {
        let _ = version.decode_initial_handshake(data);
        let _ = version.decode_auth_signature(data);
    }
});
