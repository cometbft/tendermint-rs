use crate::signature::Verifier;
use digest::FixedOutput;
use digest::{consts::U32, Digest};
use signature::{DigestSigner, DigestVerifier, Signer};

pub trait CryptoProvider {
    type Sha256: Digest + FixedOutput<OutputSize = U32>;

    type EcdsaSecp256k1Signer: Signer<k256::ecdsa::Signature>;
    type EcdsaSecp256k1Verifier: Verifier<k256::ecdsa::Signature>;

    // type Ed25519Signer: Signer<ed25519::Signature>;
    // type Ed25519Verifier: Verifier<ed25519::Signature>;
}

/// A default implementation of the HostFunctionManager that uses the [`CryptoProvider`] trait
use core::marker::PhantomData;

pub struct DefaultHostFunctionsManager;
use k256::ecdsa::{SigningKey, VerifyingKey};

#[derive(Debug, Default)]
pub struct DefaultSha256(sha2::Sha256);

pub struct DefaultSigner<D> {
    inner: SigningKey,
    _d: PhantomData<D>,
}
#[derive(Debug)]
pub struct DefaultSignatureVerifier<D> {
    inner: VerifyingKey,
    _d: PhantomData<D>,
}

impl<D: Digest + FixedOutput<OutputSize = U32>> DefaultSignatureVerifier<D> {
    fn from_bytes(public_key: &[u8]) -> Result<Self, ed25519::Error> {
        Ok(Self {
            inner: VerifyingKey::from_sec1_bytes(public_key)?,
            _d: PhantomData::default(),
        })
    }
}

impl<D: Digest + FixedOutput<OutputSize = U32>, S: signature::Signature> DigestVerifier<D, S>
    for DefaultSignatureVerifier<D>
where
    VerifyingKey: DigestVerifier<D, S>,
{
    fn verify_digest(&self, digest: D, signature: &S) -> Result<(), ed25519::Error> {
        self.inner.verify_digest(digest, signature)
    }
}

impl<S: signature::PrehashSignature, D: Digest + FixedOutput<OutputSize = U32>> Verifier<S>
    for DefaultSignatureVerifier<D>
where
    VerifyingKey: DigestVerifier<D, S>,
{
    fn verify(&self, msg: &[u8], signature: &S) -> Result<(), ed25519::Error> {
        let mut hasher = D::new();
        Digest::update(&mut hasher, msg);
        self.verify_digest(hasher, signature)
    }
}

impl digest::OutputSizeUser for DefaultSha256 {
    type OutputSize = U32;
}

impl digest::HashMarker for DefaultSha256 {}

impl digest::Update for DefaultSha256 {
    fn update(&mut self, data: &[u8]) {
        use sha2::Digest;
        self.0.update(data);
    }
}

impl FixedOutput for DefaultSha256 {
    fn finalize_into(self, out: &mut digest::Output<Self>) {
        use sha2::Digest;
        *out = self.0.finalize();
    }
}

impl<D: Digest, S: signature::Signature> Signer<S> for DefaultSigner<D>
where
    SigningKey: DigestSigner<D, S>,
{
    fn try_sign(&self, msg: &[u8]) -> Result<S, ed25519::Error> {
        let mut hasher = D::new();
        Digest::update(&mut hasher, msg);
        self.inner.try_sign_digest(hasher)
    }
}

trait DefaultHostFunctions: CryptoProvider {
    fn sha2_256(preimage: &[u8]) -> [u8; 32];
    fn ed25519_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> Result<(), ()>;
    fn secp256k1_verify(sig: &[u8], message: &[u8], public: &[u8]) -> Result<(), ()>;
}

impl CryptoProvider for DefaultHostFunctionsManager {
    type Sha256 = DefaultSha256;

    type EcdsaSecp256k1Signer = DefaultSigner<Self::Sha256>;
    type EcdsaSecp256k1Verifier = DefaultSignatureVerifier<Self::Sha256>;
}

impl DefaultHostFunctions for DefaultHostFunctionsManager {
    fn sha2_256(preimage: &[u8]) -> [u8; 32] {
        let mut hasher = Self::Sha256::new();
        hasher.update(preimage);
        hasher.finalize().try_into().unwrap()
    }

    fn ed25519_verify(sig: &[u8], msg: &[u8], pub_key: &[u8]) -> Result<(), ()> {
        let verifier =
            <<Self as CryptoProvider>::EcdsaSecp256k1Verifier>::from_bytes(pub_key).unwrap();
        let signature = k256::ecdsa::Signature::from_der(sig).unwrap();
        Ok(verifier.verify(msg, &signature).unwrap())
    }

    fn secp256k1_verify(_sig: &[u8], _message: &[u8], _public: &[u8]) -> Result<(), ()> {
        unimplemented!()
    }
}
