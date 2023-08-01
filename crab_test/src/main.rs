// #![allow(unused_imports)]

use crate::{
    content_client::XchaClient,
    opt_parse::{args_init, Cmds, UrlType},
    others::enable_log,
    page_parse::PageParser,
    splash_client::SplashClient,
};
use once_cell::sync::OnceCell;
use opt_parse::{Download, DownloadType, Parse};
use others::parse_number_range;
use std::path::PathBuf;
use tracing::{debug, error};

pub mod content_client;
pub mod content_types;
pub mod header;
pub mod opt_parse;
pub mod others;
pub mod page_parse;
pub mod splash_client;

pub static PROXY: OnceCell<Option<String>> = OnceCell::new();
pub static SAVE_DIR: OnceCell<PathBuf> = OnceCell::new();
pub static SPLASH_ADDR: OnceCell<String> = OnceCell::new();
pub static DOWNLOAD_TYPE: OnceCell<DownloadType> = OnceCell::new();

pub const XCHAIN_BASE_URL: &str = "https://xchina.co";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (simple_opts, opts) = args_init();
    enable_log();

    /*
     ┌─────────────────────────────────────────────────────────────────────────────┐
     │     配置环境                                                                 │
     └─────────────────────────────────────────────────────────────────────────────┘
    */
    {
        debug!("设置的参数: {:#?}", simple_opts);
        SPLASH_ADDR.set(simple_opts.splash_addr).unwrap();
        SAVE_DIR.set(simple_opts.save_dir).unwrap();
        PROXY.set(simple_opts.proxy).unwrap();
    }

    let start = std::time::Instant::now();

    match opts.cmds {
        Cmds::Parse(p) => parse(&p).await,
        Cmds::Download(p) => {
            DOWNLOAD_TYPE.set(p.only).unwrap();
            download(&p).await;
        }
        Cmds::No => {
            // let url = "https://xchina.co/photos/series-5f1476781eab4.html";
            // let url = "https://xchina.co/photo/id-5f55202b3e808.html";
            // let url = "https://xchina.co/photos/model-5f21a825d02c9/9.html"; // 最老的页面
            // let url = "https://xchina.co/photo/id-64a4349d31f29.html";  // + 1v
            // let url = "https://xchina.co/photo/id-64c4abcd9026b.html"; // 蒙汗药baby
            // let url = "https://xchina.co/photo/id-64c4cfb6d472f.html"; // + 3v
            // let splash_client = SplashClient::new();
            // let str = splash_client.get_html(url).await?;
            // let content = PageParser::content_urls_one_page(&str);
            // println!("{:#?}", content);
            // let page_urls = PageParser::parse_pages_urls(&str);
            // println!("{:#?}", page_urls);
            // let contents = PageParser::parse_serie_page(&str);
            // println!("{:#?}", contents);

            let splash_client = SplashClient::new();
            let page_parse = PageParser::new(splash_client);
            // let url = "https://xchina.co/photo/id-64846cdd817b7.html";
            let url = "https://xchina.co/photo/id-6496855837cde.html";
            let content = page_parse.all_content_urls(url).await;
            println!("res: {:#?}", content);

            // let url = "https://xchina.co/photos/series-5f1495dbda4de.html";
            // let client = SplashClient::new(&_splash_addr);
            // let rx = PageParser::get_all_serie_urls(&client, url).await;
            // if let Some(mut rx) = rx {
            //     while let Some((contents, page_url)) = rx.recv().await {
            //         println!("page_url: {}, len: {}", page_url, contents.len());
            //     }
            // }

            // let mut tasks = vec![];
            // for i in 20..=40 {
            //     let c = splash_client.clone();
            //     let s = start.clone();
            //     let task = tokio::spawn(async move {
            //         let url = format!("https://xchina.co/photos/series-5f1476781eab4/{}.html", i);
            //         let str = c.get_html(&url).await.unwrap();
            //         // let urls = PageParser::parse_pages_urls(&str);
            //         let _contents = PageParser::parse_serie_page(&str);
            //         // let content = PageParser::parse_content_page(&str);
            //         eprintln!("task {}, {}, len: {}", i, s.elapsed().as_secs_f64(), _contents.len());
            //     });
            //     tasks.push(task);
            // }
            // for t in tasks {
            //     t.await.unwrap();
            // }
        }
    }

    eprintln!("任务完成，用时：{}", start.elapsed().as_secs_f64());
    // tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    Ok(())
}

async fn parse(opts: &Parse) {
    let url = match UrlType::parse(&opts.url) {
        Some(s) => s,
        None => panic!("无效的url: {}", opts.url),
    };

    match &url {
        UrlType::SingleFile(u) => {
            error!("({})不是可解析页面", u);
        }
        UrlType::MainPage(u) => {
            // 获取该页
            let splash_client = SplashClient::new();
            let res = PageParser::new(splash_client).parse_main_page(u).await;
            println!("{:#?}", res);
        }
        UrlType::ZuoPing(u) => {
            // 解析页面中的所有内容列表
            let splash_client = SplashClient::new();
            let content = match PageParser::new(splash_client).all_content_urls(u).await {
                Some(c) => c,
                None => {
                    error!("无法解析该页: {}", u);
                    return;
                }
            };

            println!("{:#?}", content);
        }
        UrlType::FenLei(url) => {
            let splash_client = SplashClient::new();
            let page_parser = PageParser::new(splash_client);
            // 获取最大的页码
            if opts.max_page {
                let urls = page_parser.parse_pages_urls(&url).await;
                for (u, _) in urls {
                    println!("{}", u);
                }
                return;
            }

            let urls = match &opts.pages {
                None => vec![url.to_string()],
                Some(range_str) => make_urls_from_range(url, range_str),
            };
            debug!("将要解析的分类页: {:#?}", urls);

            let content_infos = page_parser.parse_multi_serie_pages(urls).await;
            println!("{:#?}", content_infos);
        }
    }
}

async fn download(opts: &Download) {
    let url = match UrlType::parse(&opts.url) {
        Some(s) => s,
        None => panic!("无效的url: {}", opts.url),
    };

    match &url {
        UrlType::MainPage(u) => error!("{}不是可下载的内容", u),
        UrlType::SingleFile(u) => XchaClient::download_one_item(u).await,
        UrlType::ZuoPing(u) => XchaClient::download_one_page(u).await,
        UrlType::FenLei(url) => {
            let urls = match &opts.pages {
                None => vec![url.to_string()],
                Some(range_str) => make_urls_from_range(url, range_str),
            };

            debug!("将要下载的分类页: {:#?}", urls);

            let client = XchaClient::new();
            let content_infos = client.page_parser.parse_multi_serie_pages(urls).await;

            // 下载每个作品中的内容
            client.download_multi_content_infos(content_infos).await;
            // for content_info in content_infos {
            //     client.download_from_content_info(content_info).await;
            // }
        }
    }
}

// 根据给定url，以及范围字符串，解析出范围内的所有Url
fn make_urls_from_range(url: &str, range_str: &str) -> Vec<String> {
    // 两种类型的页面，要去除base url: https://xchina.co/photos/series-5f1476781eab4
    // (1)."https://xchina.co/photos/series-5f1476781eab4.html"
    // (2)."https://xchina.co/photos/series-5f1476781eab4/1.html"

    // 移除可能的尾随斜线
    let url = url.strip_suffix('/').unwrap_or_else(|| url);

    // 两种情况：
    // left: https://xchina.co/photos, right: series-5f1476781eab4.html
    // left: https://xchina.co/photos/series-5f1476781eab4, right: 1.html
    let (left, right) = url.rsplit_once('/').unwrap();
    let filename = right.strip_suffix(".html").unwrap();

    let (current_page_num, base_url) = match filename.parse::<i16>() {
        Ok(n) => (n, left),
        Err(_) => (1, url.rsplit_once('.').unwrap().0),
    };

    // 要解析的页码
    let pages_num = parse_number_range(range_str, current_page_num);

    // 合成要解析的页码的url
    let mut urls = vec![];
    for i in pages_num {
        urls.push(format!("{}/{}.html", base_url, i));
    }

    urls
}
