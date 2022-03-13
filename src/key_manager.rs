use crate::blockchain::transaction::Address;
use crate::util;
use anyhow::Result;
use rand::rngs::OsRng;
use rsa::pkcs1::ToRsaPublicKey;
use rsa::pkcs8::ToPrivateKey;
use rsa::{Hash, PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};
use sha2::digest::Output;
use sha2::{Digest, Sha256};

pub struct KeyManager {
    private_key: RsaPrivateKey,
    public_key: RsaPublicKey,
    rng: OsRng,
}

impl KeyManager {
    pub fn new(mut rng: OsRng) -> Result<KeyManager> {
        let bits = 2048;
        let private_key = RsaPrivateKey::new(&mut rng, bits)?;
        let public_key = RsaPublicKey::from(&private_key);
        Ok(KeyManager {
            private_key,
            public_key,
            rng,
        })
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        let encrypted = self.public_key.encrypt(&mut self.rng, padding, data)?;
        Ok(encrypted)
    }

    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let padding = PaddingScheme::new_pkcs1v15_encrypt();
        let decrypted = self.private_key.decrypt(padding, data)?;
        Ok(decrypted)
    }

    pub fn sign(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        util::sign(&self.private_key, data)
    }

    /// Returns Ok(()) if verification succeeds.
    pub fn verify_signature(&mut self, data: &[u8], signature: &[u8]) -> Result<()> {
        util::verify_signature(signature, &self.public_key, data)
    }

    pub fn get_address(&self) -> Address {
        util::bytes_to_hex(self.public_key.to_pkcs1_der().unwrap().as_der())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let rng = OsRng;
        let mut km = KeyManager::new(rng).unwrap();

        let data = "abc".as_bytes();
        let signature = km.sign(data).unwrap();

        assert!(km.verify_signature(&data, &signature).is_ok());
    }
}
