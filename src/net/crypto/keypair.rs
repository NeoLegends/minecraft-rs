use bytes::Bytes;
use openssl::rsa::Rsa;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Keypair {
    pub public: Bytes,
    pub private: Bytes,
}

impl Keypair {
    pub fn generate() -> Self {
        let key = Rsa::generate(1024).expect("failed to generate RSA keypair");
        let public = key
            .public_key_to_der()
            .expect("failed to convert public key to DER");
        let private = key
            .private_key_to_der()
            .expect("failed to convert public key to DER");

        Keypair {
            public: public.into(),
            private: private.into(),
        }
    }
}
