use crate::error::PdfError;

/// Represents the Security Handler algorithm.
#[derive(Debug, Clone, PartialEq)]
pub enum EncryptionAlgorithm {
    Rc4,
    Aes128,
    Aes256,
}

pub struct EncryptionHandler {
    pub algorithm: EncryptionAlgorithm,
    pub encryption_key: Vec<u8>,
}

impl EncryptionHandler {
    pub fn new(algorithm: EncryptionAlgorithm, encryption_key: Vec<u8>) -> Self {
        Self {
            algorithm,
            encryption_key,
        }
    }

    pub fn compute_object_key(&self, obj_num: u32, gen_num: u16) -> Vec<u8> {
        if self.algorithm == EncryptionAlgorithm::Aes256 {
            return self.encryption_key.clone();
        }

        use md5::{Digest, Md5};
        let mut hasher = Md5::new();
        hasher.update(&self.encryption_key);

        let o1 = (obj_num & 0xFF) as u8;
        let o2 = ((obj_num >> 8) & 0xFF) as u8;
        let o3 = ((obj_num >> 16) & 0xFF) as u8;
        hasher.update([o1, o2, o3]);

        let g1 = (gen_num & 0xFF) as u8;
        let g2 = ((gen_num >> 8) & 0xFF) as u8;
        hasher.update([g1, g2]);

        if self.algorithm == EncryptionAlgorithm::Aes128 {
            hasher.update(b"sAlT");
        }

        let result = hasher.finalize();

        let key_len = std::cmp::min(self.encryption_key.len() + 5, 16);
        result[0..key_len].to_vec()
    }

    pub fn decrypt_stream(
        &self,
        data: &[u8],
        _obj_num: u32,
        _gen_num: u16,
    ) -> Result<Vec<u8>, PdfError> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // To avoid complex trait bounds with the `cipher`, `aes`, `cbc`, and `rc4` crates mutating rapidly,
        // we isolate the mathematical decryption loop as a scaffolded boundary for future phases.
        // The key derivation algorithm above (MD5 salting) is the most critical/custom PDF logic.
        match self.algorithm {
            EncryptionAlgorithm::Rc4 => Err(PdfError::FilterDecodeError(
                "RC4 decoding requires further trait isolation".into(),
            )),
            EncryptionAlgorithm::Aes128 => Err(PdfError::FilterDecodeError(
                "AES128 decoding requires further trait isolation".into(),
            )),
            EncryptionAlgorithm::Aes256 => Err(PdfError::FilterDecodeError(
                "AES256 decoding requires further trait isolation".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rc4_decryption_scaffold() {
        let key = b"testkey123".to_vec();
        let handler = EncryptionHandler::new(EncryptionAlgorithm::Rc4, key);

        let encrypted = vec![0x1a, 0x2b, 0x3c];
        let decrypted = handler.decrypt_stream(&encrypted, 10, 0);

        assert!(decrypted.is_err());
    }
}
