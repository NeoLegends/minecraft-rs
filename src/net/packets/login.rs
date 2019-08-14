use super::{
    bufext::{var_usize_length, VarReadExt, VarWriteExt},
    Incoming, Outgoing,
};
use bytes::{BufMut, Bytes, BytesMut};
use std::{
    convert::TryFrom,
    io::{self, Cursor, Seek, SeekFrom},
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Disconnect {
    pub reason: String,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EncryptionRequest {
    pub server_id: String,
    pub public_key: Bytes,
    pub verify_token: Bytes,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EncryptionResponse {
    pub shared_secret: Bytes,
    pub verify_token: Bytes,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LoginStart {
    pub username: String,
}

impl Outgoing for Disconnect {
    fn written_len(&self) -> usize {
        var_usize_length(self.reason.len()) + self.reason.len()
    }

    fn write_to(&self, dst: &mut BytesMut) -> io::Result<()> {
        dst.write_str(&self.reason)
    }
}

impl Outgoing for EncryptionRequest {
    fn written_len(&self) -> usize {
        var_usize_length(self.server_id.len())
            + var_usize_length(self.public_key.len())
            + var_usize_length(self.verify_token.len())
            + self.server_id.len()
            + self.public_key.len()
            + self.verify_token.len()
    }

    fn write_to(&self, dst: &mut BytesMut) -> io::Result<()> {
        let pkey_len = self.public_key.len() as i32;
        let token_len = self.verify_token.len() as i32;

        dst.write_str(&self.server_id)?;
        dst.write_var_i32(pkey_len)?;
        dst.put(&self.public_key);
        dst.write_var_i32(token_len)?;
        dst.put(&self.verify_token);

        Ok(())
    }
}

impl TryFrom<Bytes> for EncryptionResponse {
    type Error = io::Error;

    fn try_from(data: Bytes) -> io::Result<Self> {
        let mut cur = Cursor::new(&data);

        let secret_len = cur.read_var_len()?;
        let shared_secret = data.slice(cur.position() as usize, secret_len as usize);

        cur.seek(SeekFrom::Current(secret_len as i64))
            .expect("seek on cursor failed");

        let token_len = cur.read_var_len()?;
        let verify_token = data.slice(cur.position() as usize, token_len as usize);

        Ok(EncryptionResponse {
            shared_secret,
            verify_token,
        })
    }
}

impl Incoming for EncryptionResponse {}

impl TryFrom<Bytes> for LoginStart {
    type Error = io::Error;

    fn try_from(data: Bytes) -> io::Result<Self> {
        let mut cur = Cursor::new(data);
        let username = cur.read_str()?;

        Ok(LoginStart { username })
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
