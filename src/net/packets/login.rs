use super::Incoming;
use bytes::Bytes;
use openssl::{
    error::ErrorStack,
    rsa::{Padding, Rsa},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct Disconnect {
    pub reason: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct EncryptionRequest {
    pub server_id: String,
    pub public_key: Bytes,

    #[serde(with = "serde_minecraft::array")]
    pub verify_token: [u8; 16],
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct EncryptionResponse {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DecryptedEncryptionResponse {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct LoginStart {
    pub username: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub struct LoginSuccess {
    pub uuid: String,
    pub username: String,
}

impl EncryptionResponse {
    pub fn decrypt_parts(
        &self,
        private_key: &[u8],
    ) -> Result<DecryptedEncryptionResponse, ErrorStack> {
        let rsa = Rsa::private_key_from_der(private_key)?;

        let mut shared_secret_buf =
            vec![0; (rsa.size() as usize).max(self.shared_secret.len())];
        let mut verify_token_buf =
            vec![0; (rsa.size() as usize).max(self.verify_token.len())];

        rsa.private_decrypt(
            &self.shared_secret,
            &mut shared_secret_buf,
            Padding::PKCS1,
        )?;
        rsa.private_decrypt(
            &self.verify_token,
            &mut verify_token_buf,
            Padding::PKCS1,
        )?;

        Ok(DecryptedEncryptionResponse {
            shared_secret: shared_secret_buf,
            verify_token: verify_token_buf,
        })
    }
}

impl Incoming for EncryptionResponse {
    fn validate(&self) -> Result<(), String> {
        if self.shared_secret.is_empty() {
            return Err("empty shared secret".to_owned());
        } else if self.shared_secret.len() != 128 {
            return Err("shared secret len invalid".to_owned());
        } else if self.verify_token.is_empty() {
            return Err("empty verify token".to_owned());
        } else if self.verify_token.len() != 128 {
            return Err("verify token len invalid".to_owned());
        }

        Ok(())
    }
}

impl Incoming for LoginStart {
    fn validate(&self) -> Result<(), String> {
        match self.username.len() {
            0 => Err("empty username".to_owned()),
            _ => Ok(()),
        }
    }
}
