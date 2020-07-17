//! [`lite::Header`] implementation for [`block::Header`].

use crate::amino_types::{message::AminoMessage, BlockId, ConsensusVersion, TimeMsg};
use crate::lite::Height;
use crate::merkle::simple_hash_from_byte_vectors;
use crate::Hash;
use crate::{block, lite, Time};

impl lite::Header for block::Header {
    type Time = Time;

    fn height(&self) -> Height {
        self.height.value()
    }

    fn bft_time(&self) -> Time {
        self.time
    }

    fn validators_hash(&self) -> Hash {
        self.validators_hash
    }

    fn next_validators_hash(&self) -> Hash {
        self.next_validators_hash
    }

    fn hash(&self) -> Hash {
        // Note that if there is an encoding problem this will
        // panic (as the golang code would):
        // https://github.com/tendermint/tendermint/blob/134fe2896275bb926b49743c1e25493f6b24cc31/types/block.go#L393
        // https://github.com/tendermint/tendermint/blob/134fe2896275bb926b49743c1e25493f6b24cc31/types/encoding_helper.go#L9:6

        let mut fields_bytes: Vec<Vec<u8>> = Vec::with_capacity(16);
        fields_bytes.push(AminoMessage::bytes_vec(&ConsensusVersion::from(
            &self.version,
        )));
        fields_bytes.push(encode_bytes(self.chain_id.as_bytes()));
        fields_bytes.push(encode_varint(self.height.value()));
        fields_bytes.push(AminoMessage::bytes_vec(&TimeMsg::from(self.time)));
        fields_bytes.push(
            self.last_block_id
                .as_ref()
                .map_or(vec![], |id| AminoMessage::bytes_vec(&BlockId::from(id))),
        );
        fields_bytes.push(self.last_commit_hash.as_ref().map_or(vec![], encode_hash));
        fields_bytes.push(self.data_hash.as_ref().map_or(vec![], encode_hash));
        fields_bytes.push(encode_hash(&self.validators_hash));
        fields_bytes.push(encode_hash(&self.next_validators_hash));
        fields_bytes.push(encode_hash(&self.consensus_hash));
        fields_bytes.push(encode_bytes(&self.app_hash));
        fields_bytes.push(self.last_results_hash.as_ref().map_or(vec![], encode_hash));
        fields_bytes.push(self.evidence_hash.as_ref().map_or(vec![], encode_hash));
        fields_bytes.push(encode_bytes(self.proposer_address.as_bytes()));

        Hash::Sha256(simple_hash_from_byte_vectors(fields_bytes))
    }
}

fn encode_bytes(bytes: &[u8]) -> Vec<u8> {
    let bytes_len = bytes.len();
    if bytes_len > 0 {
        let mut encoded = vec![];
        prost_amino::encode_length_delimiter(bytes_len, &mut encoded).unwrap();
        encoded.append(&mut bytes.to_vec());
        encoded
    } else {
        vec![]
    }
}

fn encode_hash(hash: &Hash) -> Vec<u8> {
    encode_bytes(hash.as_bytes())
}

fn encode_varint(val: u64) -> Vec<u8> {
    let mut val_enc = vec![];
    prost_amino::encoding::encode_varint(val, &mut val_enc);
    val_enc
}

#[cfg(test)]
mod test {
    use crate::block::Header;
    use crate::lite::Header as _;
    use crate::Hash;
    use std::str::FromStr;

    #[test]
    fn test_hash_height_1() {
        // JSON extracted from https://github.com/tendermint/tendermint/tree/v0.33
        // more precisely `curl`ed from locally build docker image of:
        // git log --pretty=format:"%H" -1
        // 606d0a89ccabbd3e59cff521f9f4d875cc366ac9
        // via
        //  curl -X GET "http://localhost:26657/commit?height=1" -H  "accept: application/json" | jq .result.signed_header.header
        let json_data = r#"
{
    "version": {
      "block": "10",
      "app": "1"
    },
    "chain_id": "dockerchain",
    "height": "1",
    "time": "2020-07-09T14:24:44.7157258Z",
    "last_block_id": {
      "hash": "",
      "parts": {
        "total": "0",
        "hash": ""
      }
    },
    "last_commit_hash": "",
    "data_hash": "",
    "validators_hash": "74F2AC2B6622504D08DD2509E28CE731985CFE4D133C9DB0CB85763EDCA95AA3",
    "next_validators_hash": "74F2AC2B6622504D08DD2509E28CE731985CFE4D133C9DB0CB85763EDCA95AA3",
    "consensus_hash": "048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
    "app_hash": "",
    "last_results_hash": "",
    "evidence_hash": "",
    "proposer_address": "AD358F20C8CE80889E0F0248FDDC454595D632AE"
}"#;
        // extracted expected hash from a commit via
        // curl -X GET "http://localhost:26657/commit?height=1" -H  "accept: application/json" | jq .result.signed_header.commit.block_id.hash
        let header: Header = serde_json::from_str(json_data).unwrap();
        let got_hash = header.hash();
        let want_hash =
            Hash::from_str("F008EACA817CF6A3918CF7A6FD44F1F2464BB24D25A7EDB45A03E8783E9AB438")
                .unwrap();

        assert_eq!(got_hash, want_hash);
    }

    #[test]
    fn test_hash_height_2() {
        // JSON test-vector extracted from https://github.com/tendermint/tendermint/tree/v0.33
        // more precisely `curl`ed from locally build docker image of:
        // git log --pretty=format:"%H" -1
        // 606d0a89ccabbd3e59cff521f9f4d875cc366ac9
        // via
        //  curl -X GET "http://localhost:26657/commit?height=2" -H  "accept: application/json" | jq .result.signed_header.header
        let json_data = r#"
{
  "version": {
    "block": "10",
    "app": "1"
  },
  "chain_id": "dockerchain",
  "height": "2",
  "time": "2020-07-10T23:47:42.8655562Z",
  "last_block_id": {
    "hash": "F008EACA817CF6A3918CF7A6FD44F1F2464BB24D25A7EDB45A03E8783E9AB438",
    "parts": {
      "total": "1",
      "hash": "BF5130E879A02AC4BB83E392732ED4A37BE2F01304A615467EE7960858774E57"
    }
  },
  "last_commit_hash": "1527AF33311EB16EF9BB15B5570DE9664D6F3598BEE08231588CED89B2D8F8EA",
  "data_hash": "",
  "validators_hash": "74F2AC2B6622504D08DD2509E28CE731985CFE4D133C9DB0CB85763EDCA95AA3",
  "next_validators_hash": "74F2AC2B6622504D08DD2509E28CE731985CFE4D133C9DB0CB85763EDCA95AA3",
  "consensus_hash": "048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
  "app_hash": "0000000000000000",
  "last_results_hash": "",
  "evidence_hash": "",
  "proposer_address": "AD358F20C8CE80889E0F0248FDDC454595D632AE"
}"#;
        // extracted expected hash from a commit via
        // curl -X GET "http://localhost:26657/commit?height=2" -H  "accept: application/json" | jq .result.signed_header.commit.block_id.hash
        let header: Header = serde_json::from_str(json_data).unwrap();
        let got_hash = header.hash();
        let want_hash =
            Hash::from_str("5CFF26AAECBEEA7CBD2181D5547BB087E4DC8004960D611ECA59CFEA724AD538")
                .unwrap();

        assert_eq!(got_hash, want_hash);
    }
}
