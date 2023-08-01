use std::collections::HashMap;
use reqwest::header::{HeaderMap, self};

pub fn xchina_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("authority", "xchina.co".parse().unwrap());
    headers.insert("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".parse().unwrap());
    headers.insert("accept-language", "zh-CN,zh;q=0.6".parse().unwrap());
    headers.insert("cache-control", "no-cache".parse().unwrap());
    headers.insert(header::COOKIE, "___uniqueId=64c1d27825de1%7C6c81f46c2001125f67943d0be9cf1df2; PHPSESSID=1408e978575b74b6d25cb96650503e38; pv_punch_pc=%7B%22count%22%3A11%2C%22expiry%22%3A1690659880%7D".parse().unwrap());
    headers.insert("pragma", "no-cache".parse().unwrap());
    headers.insert(
        "referer",
        "https://xchina.co/photos/series-5f1476781eab4.html"
            .parse()
            .unwrap(),
    );
    headers.insert(
        "sec-ch-ua",
        "\"Not/A)Brand\";v=\"99\", \"Brave\";v=\"115\", \"Chromium\";v=\"115\""
            .parse()
            .unwrap(),
    );
    headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
    headers.insert("sec-ch-ua-platform", "\"Windows\"".parse().unwrap());
    headers.insert("sec-fetch-dest", "document".parse().unwrap());
    headers.insert("sec-fetch-mode", "navigate".parse().unwrap());
    headers.insert("sec-fetch-site", "same-origin".parse().unwrap());
    headers.insert("sec-fetch-user", "?1".parse().unwrap());
    headers.insert("sec-gpc", "1".parse().unwrap());
    headers.insert("upgrade-insecure-requests", "1".parse().unwrap());
    headers.insert("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36".parse().unwrap());

    headers
}

pub fn xchina_headers_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (k, v) in xchina_headers().drain() {
        map.insert(k.unwrap().to_string(), v.to_str().unwrap().to_string());
    }
    map
}
