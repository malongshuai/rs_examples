//! 使用 chacha20poly1305 可进行对称加解密(使用密码进行加密和解密)，
//! 加密时，要求密码长度和nonce长度，nonce要求唯一。
//! 明文密码可使用 argon2 进行hash化，并可填充到指定长度

use anyhow::anyhow;
use chacha20poly1305::{
    aead::{rand_core::RngCore, Aead, OsRng},
    ChaCha20Poly1305, KeyInit,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EncryptData {
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub data: Vec<u8>,
}

impl EncryptData {
    /// 加密，并使用hex进行编码得到字符串格式的加密数据
    /// 参数 data 是bincode::serialize() 得到的结果
    pub fn encrypt<T>(data: &T, passwd: &str) -> Result<String, anyhow::Error>
    where
        T: Serialize,
    {
        let data = bincode::serialize(data)?;
        let encrypt_data = Self::inner_encrypt(data, passwd)?;
        // 加密后的数据是 Vec<u8>， 如果要传递走，可考虑使用hex或者base64进行编码
        // base64编码时可能会含有特殊字符`+/`并可能在尾部填充字符`=`，
        // 如果要编码为Url，可以使用hex编码，也可以考虑使用UrlSafe的base64，考虑:
        // let encode_config = base64::Config::new(base64::CharacterSet::UrlSafe, false);
        // let en_data = base64::encode_config(&encrypt_data, encode_config);

        // Ok(base64::encode(&encrypt_data))
        Ok(hex::encode(&encrypt_data))
    }

    /// 对加密后的字符串(hex格式)进行解密
    pub fn decrypt<S>(enc_data: &str, passwd: &str) -> Result<S, anyhow::Error>
    where
        S: DeserializeOwned,
    {
        // 首先对base64编码或hex编码后的字符进行解码
        // let bincode_data = base64::decode(enc_data)?;
        let bincode_data = hex::decode(enc_data)?;
        let encrypt_data = bincode::deserialize::<Self>(&bincode_data)?;
        // 解密
        let plain_data_vec = encrypt_data.inner_decrypt(passwd)?;
        bincode::deserialize::<S>(&plain_data_vec)
            .map_err(|e| anyhow!("bincode deserialize error: {}", e))
    }

    /// 生成96bit(12bytes)的nonce
    fn gen_nonce() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        nonce
    }

    /// 生成salt，长度至少8位
    fn gen_salt() -> [u8; 8] {
        let mut salt = [0u8; 8];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    /// 给定明文指定的密码，根据 argon2 生成安全的指定长度的密码(hash之后的)
    fn gen_passwd(passwd: &str, salt: &[u8]) -> Vec<u8> {
        let config = argon2::Config {
            hash_length: 32,
            ..argon2::Config::default()
        };
        argon2::hash_raw(passwd.as_bytes(), salt, &config).unwrap()
    }

    fn inner_encrypt(data: Vec<u8>, passwd: &str) -> Result<Vec<u8>, anyhow::Error> {
        let salt = Self::gen_salt();
        let passwd_key = Self::gen_passwd(passwd, &salt);

        let key = chacha20poly1305::Key::from_slice(&passwd_key);
        let cipher = ChaCha20Poly1305::new(key);

        let nonce = Self::gen_nonce();
        let nonce = chacha20poly1305::Nonce::from_slice(&nonce);

        let cipher_ctx = cipher
            .encrypt(nonce, data.as_ref())
            .map_err(|e| anyhow!("encrypt error: {}", e))?;
        let encrypt_data = EncryptData {
            salt: salt.to_vec(),
            nonce: nonce.to_vec(),
            data: cipher_ctx,
        };
        Ok(bincode::serialize(&encrypt_data).unwrap())
    }

    fn inner_decrypt(&self, passwd: &str) -> Result<Vec<u8>, anyhow::Error> {
        let passwd_key = Self::gen_passwd(passwd, &self.salt);
        let key = chacha20poly1305::Key::from_slice(&passwd_key);
        let cipher = ChaCha20Poly1305::new(key);

        let nonce = &self.nonce;
        let nonce = chacha20poly1305::Nonce::from_slice(nonce);
        cipher
            .decrypt(nonce, self.data.as_ref())
            .map_err(|e| anyhow!("decrypt error: {}", e))
    }
}

fn main() {
    #[derive(Debug, Deserialize, Serialize)]
    struct User {
        name: String,
        addr: String,
    }

    let passwd = "thisispassword";
    let u = User {
        name: "good".into(),
        addr: "kflsad".into(),
    };

    // 加密
    let enc_data = EncryptData::encrypt(&u, passwd).unwrap();
    println!("encrypted data: {}", enc_data);

    // 解密
    let plain_data = EncryptData::decrypt::<User>(&enc_data, passwd).unwrap();
    println!("decrypted data: {:?}", plain_data);
}
