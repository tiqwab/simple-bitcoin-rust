use sha2::{Digest, Sha256};

pub fn to_hex(xs: &[u8]) -> String {
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
    to_hex(&hasher.finalize().to_vec())
}
