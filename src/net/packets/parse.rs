use super::{bufext::VarReadExt, IncomingPackets, OutgoingPackets};
use crate::net::ConnectionState;
use bytes::{BufMut, Bytes, BytesMut};
use std::io::{Cursor, Error, ErrorKind};
use tokio::codec::{Decoder, Encoder};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Coder {
    state: ConnectionState,
}

impl Coder {
    pub fn new(state: ConnectionState) -> Self {
        Coder { state }
    }

    pub fn state(&self) -> ConnectionState {
        self.state
    }

    pub fn set_state(&mut self, new_state: ConnectionState) {
        self.state = new_state;
    }
}

macro_rules! eof_to_none {
    ($val:expr) => {
        match $val {
            Ok(v) => Ok(v),
            Err(ref err) if err.kind() == ::std::io::ErrorKind::UnexpectedEof => {
                return Ok(None)
            }
            Err(e) => Err(e),
        }
    };
}

macro_rules! parse_table {
    ($packetid:expr, $contents:expr, $($pid:expr => $type:ident),*) => {
        match $packetid {
            $($pid => {
                let packet = ::std::convert::TryInto::try_into($contents)?;
                $crate::net::packets::IncomingPackets::$type(packet)
            },)*
            _ => return Err(
                ::std::io::Error::new(::std::io::ErrorKind::InvalidData,
                "unexpected packet ID",
            ))
        }
    };
}

impl Coder {
    fn read_chunk(src: &mut BytesMut) -> Result<Option<(i32, Bytes)>, Error> {
        let mut cur = Cursor::new(src.by_ref());

        let length_with_pid = eof_to_none!(cur.read_var_len())?;
        let length_of_len_field = cur.position() as usize;

        let packet_id = eof_to_none!(cur.read_var_i32())?;
        let length_of_pid_field = cur.position() as usize - length_of_len_field;

        let total_length = length_with_pid + length_of_len_field;

        if src.len() < total_length {
            return Ok(None);
        }

        let contents_length = length_with_pid - length_of_pid_field;

        src.advance(length_of_len_field + length_of_pid_field);
        let contents_data = src.split_to(contents_length).freeze();

        Ok(Some((packet_id, contents_data)))
    }
}

impl Decoder for Coder {
    type Item = IncomingPackets;
    type Error = Error;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        use ConnectionState::*;

        let parsed = match Self::read_chunk(src)? {
            Some((packet_id, data)) => match self.state() {
                Start => parse_table!(
                    packet_id,
                    data,
                    0 => Handshake
                ),
                Login => parse_table!(
                    packet_id,
                    data,
                    0 => LoginStart,
                    1 => EncryptionResponse
                ),
                Play => unimplemented!(),
                Status => parse_table!(
                    packet_id,
                    data,
                    0 => StatusHandshake,
                    1 => Ping
                ),
            },
            None => return Ok(None),
        };

        Ok(Some(parsed))
    }
}

macro_rules! serialize_table {
    ($item:expr, $dst:expr, $($packet:ident => $packet_id:expr),+) => {
        match $item {
            $(OutgoingPackets::$packet(p) => {
                use $crate::net::packets::{
                    Outgoing,
                    bufext::VarWriteExt,
                };

                let packet_id_len = $crate::net::packets::bufext::var_i32_length($packet_id);
                let total_len = p.written_len() + packet_id_len;

                $dst.reserve(total_len);

                $dst.write_var_i32(total_len as i32)?;
                $dst.write_var_i32($packet_id)?;
                p.write_to($dst)?;
            })*,
            _ => return Err(::std::io::Error::new(
                ::std::io::ErrorKind::Other,
                "unexpected packet for state"
            )),
        }
    };
}

impl Encoder for Coder {
    type Item = OutgoingPackets;
    type Error = Error;

    fn encode(
        &mut self,
        item: Self::Item,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        match self.state() {
            ConnectionState::Start => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "unexpected packet for start state",
                ));
            }
            ConnectionState::Login => serialize_table!(
                item,
                dst,
                Disconnect => 0,
                EncryptionRequest => 1
            ),
            ConnectionState::Play => unimplemented!(),
            ConnectionState::Status => serialize_table!(
                item,
                dst,
                StatusResponse => 0,
                Ping => 1
            ),
        }

        Ok(())
    }
}
