//! `intialize` subcommand

use crate::application::app_config;
use crate::config::LightClientConfig;
use abscissa_core::{Command, Options, Runnable};
use std::collections::HashMap;
use tendermint::{hash, Hash};

use tendermint::lite::Header;

use tendermint_light_client::components::io::{AtHeight, Io, ProdIo};
use tendermint_light_client::operations::ProdHasher;
use tendermint_light_client::predicates::{ProdPredicates, VerificationPredicates};
use tendermint_light_client::store::sled::SledStore;
use tendermint_light_client::store::LightStore;
use tendermint_light_client::types::Status;

/// `intialize` subcommand
#[derive(Command, Debug, Default, Options)]
pub struct InitCmd {
    #[options(
        free,
        help = "subjective height of the initial trusted state to initialize the node with"
    )]
    pub height: u64,

    #[options(
        free,
        help = "hash of the initial subjectively trusted header to initialize the node with"
    )]
    pub header_hash: String,
}

impl Runnable for InitCmd {
    fn run(&self) {
        let subjective_header_hash = Hash::from_hex_upper(hash::Algorithm::Sha256, &self.header_hash).unwrap();
        let app_cfg = app_config();

        let lc = app_cfg.light_clients.first().unwrap();

        let mut peer_map = HashMap::new();
        peer_map.insert(lc.peer_id, lc.address.clone());

        let io = ProdIo::new(peer_map, Some(app_cfg.request_timeout));

        initialize_subjectively(self.height, subjective_header_hash, &lc, &io);
    }
}

fn initialize_subjectively(
    height: u64,
    subjective_header_hash: Hash,
    l_conf: &LightClientConfig,
    io: &ProdIo,
) {
    let db = sled::open(l_conf.db_path.clone()).unwrap_or_else(|e| {
        println!("[ error ] could not open database: {}", e);
        std::process::exit(1);
    });

    let mut light_store = SledStore::new(db);

    if light_store.latest(Status::Verified).is_some() {
        println!("[ warning ] overwriting trusted state in database");
    }

    let trusted_state = io
        .fetch_light_block(l_conf.peer_id, AtHeight::At(height))
        .unwrap_or_else(|e| {
            println!("[error] could not retrieve trusted header: {}", e);
            std::process::exit(1);
        });

    let predicates = ProdPredicates;
    let hasher = ProdHasher;
    if let Err(err) = predicates.validator_sets_match(&trusted_state, &hasher) {
        println!("[error] invalid light block: {}", err);
        std::process::exit(1);
    }
    // TODO(ismail): actually verify more predicates of light block before storing!?
    let got_header_hash = trusted_state.signed_header.header.hash();
    if got_header_hash != subjective_header_hash {
        println!(
            "[error] received LightBlock's header hash: {} does not match the subjective hash: {}",
            got_header_hash, subjective_header_hash
        );
        std::process::exit(1);
    }
    light_store.insert(trusted_state, Status::Verified);
}
