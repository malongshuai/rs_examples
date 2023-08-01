//! 向Splash发送请求HTML页面
//!

use crate::{header::xchina_headers_map, SPLASH_ADDR};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use tracing::{debug, instrument};

/// splash服务端的地址和端口
const SPLASH_URL: &str = "http://192.168.200.8:8050/render.html";

/// splash要使用的proxy
const SPLASH_PROXY: &str = "http://192.168.200.1:8118";

#[derive(Debug, Serialize)]
pub struct SplashPostData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    pub images: u8,
    pub headers: HashMap<String, String>,
    pub timeout: u16,
}

impl Default for SplashPostData {
    fn default() -> Self {
        Self {
            proxy: Some(SPLASH_PROXY.to_string()),
            // proxy: None,
            images: 0,
            headers: xchina_headers_map(),
            timeout: 60,
        }
    }
}

/// 发送请求给Splash服务端的客户端，通过post请求Splash，可以传递更多数据给Splash
#[derive(Clone)]
pub struct SplashClient {
    pub conn: reqwest::Client,
    splash_url: Arc<String>,
    splash_data: Arc<SplashPostData>,
}

impl SplashClient {
    pub fn new() -> Self {
        let conn = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();

        let splash_addr = SPLASH_ADDR.get().unwrap();
        let splash_addr = match splash_addr.starts_with("http") {
            true => format!("{}/render.html", splash_addr),
            false => format!("http://{}/render.html", splash_addr),
        };

        Self {
            conn,
            splash_url: Arc::new(splash_addr),
            splash_data: Arc::new(SplashPostData::default()),
        }
    }

    /// 向Splash发送获取html的请求，会重试最多三次
    pub async fn get_html_retry(&self, url: &str) -> Result<String, reqwest::Error> {
        for _ in 1..3 {
            if let Ok(resp) = self.get_html(url).await {
                // 有可能请求正确，但是返回超时渲染消息 
                // {"error": 504, "type": "GlobalTimeoutError", "description": "Timeout exceeded rendering page", "info": {"remaining": -0.001005, "timeout": 30}}
                if !resp.contains(r##"{"error": 504"##) {
                    return Ok(resp);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        self.get_html(url).await
    }

    #[instrument(skip(self))]
    pub async fn get_html(&self, url: &str) -> Result<String, reqwest::Error> {
        // 构建splash要代理请求的地址
        let req_url = format!("{}?url={}", self.splash_url, url);
        debug!(
            "send request: {}, post_data: {:?}",
            req_url, self.splash_data
        );
        let req = self.conn.post(req_url).json(&*self.splash_data);
        let res = req.send().await?.text().await?;

        Ok(res)
    }
}

impl SplashClient {
    pub async fn get_html_simple(&self, url: &str) -> Result<String, reqwest::Error> {
        let url = Self::make_simpl_splash_url(url);
        debug!("send_req: {}", url);
        let res = self.conn.get(url).send().await?.text().await?;
        Ok(res)
    }

    /// 给定一个URL，创建splash的访问url
    fn make_simpl_splash_url(url: &str) -> String {
        format!("{}?proxy={}&images=0&url={}", SPLASH_URL, SPLASH_PROXY, url)
    }
}
