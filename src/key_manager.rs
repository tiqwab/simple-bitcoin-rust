use crate::blockchain::transaction::Address;
use crate::util;
use anyhow::Result;
use rand::rngs::OsRng;
use rsa::pkcs1::ToRsaPublicKey;
use rsa::pkcs8::ToPrivateKey;
use rsa::{Hash, PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};
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
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hashed = hasher.finalize();

        let padding = PaddingScheme::PKCS1v15Sign {
            hash: Some(Hash::SHA2_256),
        };
        let signed = self.private_key.sign(padding, &hashed)?;

        Ok(signed)
    }

    /// Returns Ok(()) if verification succeeds.
    pub fn verify_signature(&mut self, hashed: &[u8], signature: &[u8]) -> Result<()> {
        let padding = PaddingScheme::PKCS1v15Sign {
            hash: Some(Hash::SHA2_256),
        };
        self.public_key.verify(padding, hashed, signature)?;
        Ok(())
    }

    pub fn get_address(&self) -> Address {
        util::to_hex(self.public_key.to_pkcs1_der().unwrap().as_der())
    }
}
