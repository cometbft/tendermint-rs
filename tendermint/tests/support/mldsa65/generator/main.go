// Generates ML-DSA-65 fixtures for tendermint-rs from CometBFT v0.40.0.
//
// Output (stdout): one JSON document with
//   - key: deterministic key material, encodings and a raw signature
//   - light_block: a signed header + validator set (one ML-DSA-65 and one
//     Ed25519 validator, both signing) that Go's VerifyCommitLight accepts.
package main

import (
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"time"

	"github.com/cloudflare/circl/sign/mldsa/mldsa65"

	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/ed25519"
	"github.com/cometbft/cometbft/crypto/encoding"
	cmtmldsa "github.com/cometbft/cometbft/crypto/mldsa65"
	"github.com/cometbft/cometbft/crypto/tmhash"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
	cmtversion "github.com/cometbft/cometbft/proto/tendermint/version"
	"github.com/cometbft/cometbft/types"
	"github.com/cometbft/cometbft/version"
)

func must(err error) {
	if err != nil {
		panic(err)
	}
}

func rawJSON(v any) json.RawMessage {
	bz, err := cmtjson.Marshal(v)
	must(err)
	return bz
}

func main() {
	const chainID = "mldsa-fixture"
	seed := make([]byte, cmtmldsa.SeedSize)
	for i := range seed {
		seed[i] = byte(i + 1)
	}
	mlPriv, err := cmtmldsa.GenPrivKeyFromSeed(seed)
	must(err)
	mlPub := mlPriv.PubKey()
	edPriv := ed25519.GenPrivKeyFromSecret([]byte("tendermint-rs mldsa fixture"))
	edPub := edPriv.PubKey()

	// --- key-level fixture ---
	pbKey, err := encoding.PubKeyToProto(mlPub)
	must(err)
	pbKeyBz, err := pbKey.Marshal()
	must(err)
	msg := []byte("tendermint-rs ml-dsa-65 fixture message")
	sig, err := mlPriv.Sign(msg)
	must(err)
	if !mlPub.VerifySignature(msg, sig) {
		panic("self-verify failed")
	}
	// Same message signed under a non-empty context: must NOT verify as pure ML-DSA.
	var sk mldsa65.PrivateKey
	must(sk.UnmarshalBinary(mlPriv.Bytes()))
	ctxSig := make([]byte, mldsa65.SignatureSize)
	must(mldsa65.SignTo(&sk, msg, []byte("ctx"), false, ctxSig))
	if mlPub.VerifySignature(msg, ctxSig) {
		panic("context signature unexpectedly verified")
	}

	// --- light block fixture ---
	valSet := types.NewValidatorSet([]*types.Validator{
		types.NewValidator(mlPub, 10),
		types.NewValidator(edPub, 10),
	})
	privs := map[string]crypto.PrivKey{
		string(mlPub.Address()): mlPriv,
		string(edPub.Address()): edPriv,
	}
	ts := time.Date(2026, 11, 11, 0, 0, 0, 0, time.UTC)
	header := types.Header{
		Version:            cmtversion.Consensus{Block: version.BlockProtocol},
		ChainID:            chainID,
		Height:             1,
		Time:               ts,
		ValidatorsHash:     valSet.Hash(),
		NextValidatorsHash: valSet.Hash(),
		ConsensusHash:      tmhash.Sum([]byte("consensus")),
		AppHash:            tmhash.Sum([]byte("app")),
		ProposerAddress:    valSet.Proposer.Address,
	}
	blockID := types.BlockID{
		Hash:          header.Hash(),
		PartSetHeader: types.PartSetHeader{Total: 1, Hash: tmhash.Sum([]byte("parts"))},
	}
	commit := &types.Commit{Height: 1, Round: 0, BlockID: blockID}
	for i, val := range valSet.Validators {
		vote := &types.Vote{
			Type:             cmtproto.PrecommitType,
			Height:           1,
			Round:            0,
			BlockID:          blockID,
			Timestamp:        ts.Add(time.Duration(i+1) * time.Second),
			ValidatorAddress: val.Address,
			ValidatorIndex:   int32(i),
		}
		s, err := privs[string(val.Address)].Sign(types.VoteSignBytes(chainID, vote.ToProto()))
		must(err)
		commit.Signatures = append(commit.Signatures, types.CommitSig{
			BlockIDFlag:      types.BlockIDFlagCommit,
			ValidatorAddress: val.Address,
			Timestamp:        vote.Timestamp,
			Signature:        s,
		})
	}
	must(valSet.VerifyCommitLight(chainID, blockID, 1, commit))
	must(valSet.VerifyCommit(chainID, blockID, 1, commit))

	out := map[string]any{
		"generator": "cometbft v0.40.0 (tendermint-rs ml-dsa-65 fixture)",
		"key": map[string]any{
			"seed_hex":           hex.EncodeToString(seed),
			"pub_key_hex":        hex.EncodeToString(mlPub.Bytes()),
			"address_hex":        hex.EncodeToString(mlPub.Address()),
			"pub_key_json":       rawJSON(mlPub),
			"pub_key_proto_hex":  hex.EncodeToString(pbKeyBz),
			"pub_key_proto_json": rawJSON(pbKey),
			"msg_hex":            hex.EncodeToString(msg),
			"sig_hex":            hex.EncodeToString(sig),
			"ctx_sig_hex":        hex.EncodeToString(ctxSig),
			"validator_set_hash": hex.EncodeToString(valSet.Hash()),
		},
		"light_block": map[string]any{
			"chain_id":      chainID,
			"signed_header": rawJSON(types.SignedHeader{Header: &header, Commit: commit}),
			"validator_set": rawJSON(valSet),
		},
	}
	enc := json.NewEncoder(os.Stdout)
	enc.SetIndent("", "  ")
	must(enc.Encode(out))
	fmt.Fprintln(os.Stderr, "ok: Go VerifyCommitLight and VerifyCommit accepted the fixture")
}
