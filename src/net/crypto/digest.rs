use openssl::sha::Sha1;

pub fn server_digest(public_key: &[u8], shared_secret: &[u8]) -> String {
    let digest = {
        let mut sha = Sha1::new();

        sha.update(shared_secret);
        sha.update(public_key);

        sha.finish()
    };

    to_minecraft_digest(digest)
}

fn to_minecraft_digest(mut hash: [u8; 20]) -> String {
    let negative = (hash[0] & 0x80) == 0x80;
    if negative {
        two_complement(&mut hash);
    }

    let digest = hash
        .into_iter()
        .map(|v| format!("{:02x}", *v))
        .collect::<String>()
        .trim_start_matches('0')
        .to_owned();

    if negative {
        format!("-{}", digest)
    } else {
        digest
    }
}

fn two_complement(bytes: &mut [u8]) {
    let mut carry = true;
    for i in (0..bytes.len()).rev() {
        bytes[i] = !bytes[i] & 0xff;
        if carry {
            carry = bytes[i] == 0xff;
            bytes[i] = bytes[i] + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minecraft_digest_notch() {
        let mut sha = Sha1::new();
        sha.update(b"Notch");

        assert_eq!(
            super::to_minecraft_digest(sha.finish()),
            "4ed1f46bbe04bc756bcb17c0c7ce3e4632f06a48",
        );
    }

    #[test]
    fn minecraft_digest_jeb() {
        let mut sha = Sha1::new();
        sha.update(b"jeb_");

        assert_eq!(
            super::to_minecraft_digest(sha.finish()),
            "-7c9d5b0044c130109a5d7b5fb5c317c02b4e28c1",
        );
    }

    #[test]
    fn minecraft_digest_simon() {
        let mut sha = Sha1::new();
        sha.update(b"simon");

        assert_eq!(
            super::to_minecraft_digest(sha.finish()),
            "88e16a1019277b15d58faf0541e11910eb756f6",
        );
    }
}
