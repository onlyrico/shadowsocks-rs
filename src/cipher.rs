use openssl::symm;
use util::generate_key;
use rand::{thread_rng, Rng};

pub struct Cipher {
    pub cipher: symm::Cipher,
    pub key: Vec<u8>,
    pub iv: Vec<u8>,
    pub iv_len: usize,
    pub enc: Option<symm::Crypter>,
    pub dec: Option<symm::Crypter>,
}

impl Cipher {
    pub fn new(method: &str, password: &str) -> Cipher {
        let cipher = match method {
            "aes-256-cfb" => symm::Cipher::aes_256_cfb128(),
            _ => panic!("method not supported"),
        };

        let key = generate_key(password.as_bytes(), cipher.key_len());
        Cipher {
            cipher: cipher,
            key: Vec::from(&key[..]),
            iv: vec![],
            iv_len: cipher.iv_len().unwrap_or_default(),
            enc: None,
            dec: None,
        }
    }

    pub fn init_encrypt(&mut self) {
        if self.iv.is_empty() {
            let mut rng = thread_rng();
            self.iv = rng.gen_iter::<u8>().take(self.iv_len).collect::<Vec<u8>>();
        }
        self.enc = Some(
            symm::Crypter::new(
                self.cipher.clone(),
                symm::Mode::Encrypt,
                &self.key,
                Some(&self.iv),
            ).expect("init enc error"),
        );
    }

    pub fn init_decrypt(&mut self, iv: &[u8]) {
        self.dec = Some(
            symm::Crypter::new(
                self.cipher.clone(),
                symm::Mode::Decrypt,
                &self.key,
                Some(iv),
            ).expect("init enc error"),
        );
    }

    pub fn encrypt(&mut self, input: &[u8]) -> Option<Vec<u8>> {
        let reserve_len = input.len() + self.cipher.block_size();
        let mut out = Vec::with_capacity(reserve_len);
        unsafe {
            out.set_len(reserve_len);
        }
        if let Some(ref mut enc) = self.enc {
            if enc.update(input, &mut out).is_ok() {
                return Some(out);
            }
        }
        return None;
    }

    pub fn decrypt(&mut self, input: &[u8]) -> Option<Vec<u8>> {
        let reserve_len = input.len() + self.cipher.block_size();
        let mut out = Vec::with_capacity(reserve_len);
        unsafe {
            out.set_len(reserve_len);
        }

        if let Some(ref mut dec) = self.dec {
            if dec.update(input, &mut out).is_ok() {
                return Some(out);
            }
        }
        return None;
    }

    pub fn reset(&self) -> Cipher {
        Cipher {
            cipher: self.cipher.clone(),
            key: self.key.clone(),
            iv: vec![],
            iv_len: self.iv_len,
            enc: None,
            dec: None,
        }
    }
}
