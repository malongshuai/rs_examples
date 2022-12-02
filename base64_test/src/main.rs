//! base64编码
//! 标准实现: 会将数据编码成`[0-9a-zA-Z+/]`共64个字符，有时候会在尾部填充 `=` 符号
//! UrlSafe实现: 会将数据编码成`[0-9a-zA-Z-_]`共64个字符，有时候会在尾部填充 `=` 符号
//! 更多实现，参考https://docs.rs/base64/latest/base64/enum.CharacterSet.html

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User<'a> {
    name: &'a str,
    addr: &'a str,
    long: bool,
    good: Option<String>,
}

fn main() {
    let data = r#"hello world 你好不好的 23891+-./"#;

    // 直接编码解码
    let en_data = base64::encode(data);
    println!("en_data: {}", en_data);
    let de_data = base64::decode(en_data.as_bytes()).unwrap();
    println!("de_data: {}", String::from_utf8_lossy(&de_data));

    let data = User {
        name: "goodboy",
        addr: "slkajdf",
        long: true,
        good: Some("akls".to_string()),
    };
    let data = bincode::serialize(&data).unwrap();

    // 使用特定 实现 进行编码
    // 明确指定不要在尾部填充 `=`
    let encode_config = base64::Config::new(base64::CharacterSet::UrlSafe, false);
    let en_data = base64::encode_config(data, encode_config);
    println!("en_data: {}", en_data);
    
    let de_data = base64::decode_config(en_data.as_bytes(), encode_config).unwrap();
    println!("de_data: {:?}", bincode::deserialize::<User>(&de_data));
}
