//! 获取内容的客户端，直接向xchina请求数据(例如，请求图片、视频)，而不是向Splash请求
//!
use crate::{
    content_types::{Content, ContentInfo},
    header::xchina_headers,
    opt_parse::DownloadType,
    page_parse::PageParser,
    splash_client::SplashClient,
    DOWNLOAD_TYPE, PROXY, SAVE_DIR,
};
use bytes::Bytes;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::{mpsc, Semaphore};
use tracing::{debug, error, info, warn};

type UUrl = String;

#[derive(Clone)]
pub struct XchaClient {
    pub conn: reqwest::Client,
    pub splash_conn: SplashClient,
    pub page_parser: PageParser,
}

impl XchaClient {
    pub fn new() -> Self {
        let mut builder = reqwest::Client::builder()
            .default_headers(xchina_headers())
            .redirect(reqwest::redirect::Policy::none());
        if let Some(Some(p)) = PROXY.get() {
            builder = builder.proxy(reqwest::Proxy::all(p).unwrap());
        }

        let conn = builder.build().unwrap();
        let splash_conn = SplashClient::new();

        let page_parser = PageParser::new(splash_conn.clone());

        Self {
            conn,
            splash_conn,
            page_parser,
        }
    }

    /// 下载作品中的内容
    pub async fn download_content(&self, content: Content) {
        let content_info = content.content_info().clone();
        let mut urls = content.urls();

        if urls.is_empty() {
            warn!("{}页没有内容可下载", content_info.page_url);
            return;
        }

        // 先拿一个url进行探测该url是否正确，如果正确，则继续，否则解析作品页获得正确的url
        let first_url = urls.first().unwrap();
        if self.download_one_retry(&first_url).await.is_err() {
            let all_content_urls = self
                .page_parser
                .all_content_urls(&content_info.page_url)
                .await;
            match all_content_urls {
                Some(c) => {
                    urls = c.urls();
                }
                None => return,
            }
        }

        if let Some(t) = DOWNLOAD_TYPE.get() {
            match t {
                DownloadType::All => {}
                DownloadType::Imgs => urls.retain(|x| !x.ends_with(".mp4")),
                DownloadType::Videos => urls.retain(|x| x.ends_with(".mp4")),
            }
        }

        debug!("等待被下载的url列表: {:#?}", urls);

        let (tx, rx) = mpsc::channel::<(Bytes, UUrl, PathBuf)>(1000);

        let save_dir = SAVE_DIR.get().unwrap();
        let save_dir = content_info.file_dir(save_dir);
        if let Err(e) = tokio::fs::create_dir_all(&save_dir).await {
            error!("创建目录 {} 失败, 错误信息: {}", save_dir.display(), e);
            return;
        }

        let s_self = self.clone();
        tokio::spawn(async move {
            let semaphore = Arc::new(Semaphore::new(20));
            let mut tasks = vec![];
            for url in urls {
                let filename = url.rsplit_once('/').unwrap().1;
                let file_path = save_dir.join(filename);
                if file_path.exists() {
                    info!("文件已存在, {}", file_path.display());
                    continue;
                }

                let s_self = s_self.clone();
                let sem = semaphore.clone();
                let tx = tx.clone();
                let task = tokio::spawn(async move {
                    let _permit = sem.acquire().await.unwrap();
                    debug!("下载 {}", url);
                    match s_self.download_one_retry(&url).await {
                        Ok(bs) => {
                            debug!("下载 {} 长度: {}", url, bs.len());
                            tx.send((bs, url, file_path)).await.unwrap();
                        }
                        Err(e) => {
                            error!("下载({})失败: {}", url, e);
                        }
                    }
                });
                tasks.push(task);
            }

            for task in tasks {
                task.await.unwrap();
            }
            drop(tx);
        });

        self.write_file(rx).await;
    }

    /// 给定一个作品基本信息，下载该作品中的所有内容(将先解析页面)
    pub async fn download_from_content_info(&self, content_info: ContentInfo) {
        let page_url = &content_info.page_url;
        // 解析页面中的所有内容列表
        let all_content_urls = self.page_parser.all_content_urls(&page_url).await;
        let content = match all_content_urls {
            Some(c) => c,
            None => {
                error!("无法解析该页: {}", page_url);
                return;
            }
        };
        debug!("解析页({})获得信息: {:?}", page_url, content);

        self.download_content(content).await;
    }

    /// 并发下载多个作品
    pub async fn download_multi_content_infos(&self, content_infos: Vec<ContentInfo>) {
        let semaphore = Arc::new(Semaphore::new(10));
        let mut tasks = vec![];

        for content_info in content_infos {
            let c_self = self.clone();
            let sem = semaphore.clone();
            let task = tokio::spawn(async move {
                let _ = sem.acquire().await.unwrap();
                c_self.download_from_content_info(content_info).await;
            });
            tasks.push(task);
        }

        for task in tasks {
            let _ = task.await;
        }
    }

    async fn download_one(&self, url: &str) -> Result<Bytes, reqwest::Error> {
        self.conn.get(url).send().await?.bytes().await
    }

    /// 重试3次的下载
    async fn download_one_retry(&self, url: &str) -> Result<Bytes, reqwest::Error> {
        for _ in 1..=2 {
            if let Ok(data) = self.download_one(url).await {
                return Ok(data);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        self.download_one(url).await
    }

    /// 阻塞等待接收下载的数据，并写入文件
    async fn write_file(&self, mut rx: mpsc::Receiver<(Bytes, UUrl, PathBuf)>) {
        while let Some((data, url, file_path)) = rx.recv().await {
            debug!("接收 {} 字节数据长度: {}", data.len(), url);
            if let Err(e) = tokio::fs::write(&file_path, data).await {
                error!("数据写入 {} 文件失败, 错误信息: {}", file_path.display(), e);
            }
            info!("下载成功: {}, 保存在: {}", url, file_path.display());
        }
    }
}

impl XchaClient {
    /// 只下载一个指定的文件，例如`https://img.xchina.biz/photos/64c4abcd9026b/0001.jpg`
    pub async fn download_one_item(url: &str) {
        let client = Self::new();

        let filename = url.rsplit_once('/').unwrap().1;
        let path = SAVE_DIR.get().unwrap().join(filename);

        match client.download_one_retry(url).await {
            Ok(data) => tokio::fs::write(&path, data).await.unwrap(),
            Err(e) => {
                error!("下载失败({})，错误信息: {}", url, e);
            }
        }
        info!("下载成功: {}，保存在 {}", url, path.display());
    }

    /// 只下载一个作品页面中的所有内容，例如：https://xchina.co/photo/id-64c4abcd9026b/1.html
    pub async fn download_one_page(url: &str) {
        let client = Self::new();

        // 解析页面中的所有内容列表
        let content = match client.page_parser.all_content_urls(url).await {
            Some(c) => c,
            None => {
                error!("无法解析该页: {}", url);
                return;
            }
        };
        debug!("解析页({})获得信息: {:#?}", url, content);

        client.download_content(content).await;
    }
}
