use crate::blockchain::transaction::{Address, TransactionSignature};
use crate::key_manager;
use anyhow::Result;
use rsa::pkcs1::FromRsaPublicKey;
use rsa::{Hash, PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};
use sha2::{Digest, Sha256};

const SIGN_PADDING_SCHEMA: PaddingScheme = PaddingScheme::PKCS1v15Sign {
    hash: Some(Hash::SHA2_256),
};

pub fn bytes_to_hex(xs: &[u8]) -> String {
    fn hex(x: u8) -> [char; 2] {
        let chars = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
        ];
        let mut buf = ['\0'; 2];
        buf[0] = chars[(x / 16) as usize];
        buf[1] = chars[(x % 16) as usize];
        buf
    }

    xs.iter().map(|x| hex(*x)).flatten().collect()
}

pub fn sha256(data: &[u8], nonce: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.update(nonce);
    bytes_to_hex(&hasher.finalize().to_vec())
}

pub fn hex_to_bytes(xs: String) -> Vec<u8> {
    fn h_to_n(x: u8) -> Option<u8> {
        if b'a' <= x && b'f' >= x {
            Some(x - b'a' + 10)
        } else if b'0' <= x && b'9' >= x {
            Some(x - b'0')
        } else {
            None
        }
    }

    assert_eq!(xs.len() % 2, 0);

    let mut res = vec![];
    let upper = xs
        .bytes()
        .enumerate()
        .filter_map(|(i, x)| if i % 2 == 0 { Some(x) } else { None });
    let lower = xs
        .bytes()
        .enumerate()
        .filter_map(|(i, x)| if i % 2 == 1 { Some(x) } else { None });
    upper.zip(lower).for_each(|(u, l)| {
        res.push(h_to_n(u).unwrap() * 16 + h_to_n(l).unwrap());
    });
    res
}

pub fn calc_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn sign(private_key: &RsaPrivateKey, data: &[u8]) -> Result<Vec<u8>> {
    let hashed = calc_hash(data);
    let signed = private_key.sign(SIGN_PADDING_SCHEMA, &hashed)?;
    Ok(signed)
}

pub fn verify_signature(signature: &[u8], public_key: &RsaPublicKey, data: &[u8]) -> Result<()> {
    let hashed = calc_hash(data);
    public_key.verify(SIGN_PADDING_SCHEMA, &hashed, signature)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_hex() {
        assert_eq!(bytes_to_hex(&[65, 98, 78]), "41624e".to_string());
    }

    #[test]
    fn test_to_bytes() {
        assert_eq!(&hex_to_bytes("41624e".to_string()), &[65, 98, 78]);
    }
}
