//! LightNode Config
//!
//! See instructions in `commands.rs` to specify the path to your
//! application's configuration file and/or command-line options
//! for specifying it.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// LightNode Configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LightNodeConfig {
    /// RPC address to request headers and validators from.
    pub rpc_address: String,
    /// The duration until we consider a trusted state as expired.
    pub trusting_period: Duration,
    /// Subjective initialization.
    pub subjective_init: SubjectiveInit,
}

/// Default configuration settings.
///
/// Note: if your needs are as simple as below, you can
/// use `#[derive(Default)]` on LightNodeConfig instead.
impl Default for LightNodeConfig {
    fn default() -> Self {
        Self {
            rpc_address: "localhost:26657".to_owned(),
            trusting_period: Duration::new(6000, 0),
            subjective_init: SubjectiveInit::default(),
        }
    }
}

/// Configuration for subjective initialization.
///
/// Contains the subjective height and validators hash (as a string formatted as hex).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SubjectiveInit {
    /// Subjective height.
    pub height: u64,
    /// Subjective validators hash.
    pub validators_hash: String,
}

impl Default for SubjectiveInit {
    fn default() -> Self {
        Self {
            height: 1,
            // TODO(liamsi): a default hash here does not make sense unless it is a valid hash
            // from a public network
            validators_hash: "A5A7DEA707ADE6156F8A981777CA093F178FC790475F6EC659B6617E704871DD"
                .to_owned(),
        }
    }
}
