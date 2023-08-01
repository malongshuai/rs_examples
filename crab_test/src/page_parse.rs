use crate::{
    content_types::{Content, ContentInfo, MainPageFenLei, Video},
    splash_client::SplashClient,
};
use scraper::{ElementRef, Html, Selector};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, error};
use url::Url;

#[derive(Clone)]
pub struct PageParser {
    splash_client: SplashClient,
}

impl PageParser {
    pub fn new(splash_client: SplashClient) -> Self {
        Self { splash_client }
    }

    pub async fn get_html(&self, url: &str) -> Option<String> {
        match self.splash_client.get_html_retry(url).await {
            Ok(s) => Some(s),
            Err(e) => {
                error!("请求({})失败, 错误信息: {}", url, e);
                return None;
            }
        }
    }

    /// 解析主页侧边栏，得到各分类信息及URL
    ///
    /// 调用该方法后，选择需要解析的分类，请求该分类的url得到html响应，
    /// 再调用`parse_serie_page_urls()`方法获取该分类中的所有作品的页码信息(包括URL)
    pub async fn parse_main_page(&self, url: &str) -> MainPageFenLei {
        let mut self_tmp = MainPageFenLei::default();

        let html_str = match self.get_html(url).await {
            Some(s) => s,
            None => return self_tmp,
        };

        let main_page_doc = Html::parse_document(&html_str);
        // 侧边栏
        let aside_series_selector = Selector::parse("div.section div.aside div.series").unwrap();

        let h3_selector = Selector::parse("h3").unwrap();
        //~ 写针分类
        {
            let mut aside_series = main_page_doc.select(&aside_series_selector);
            let series = aside_series
                .find(|ele| {
                    let h3_str = match ele.select(&h3_selector).next() {
                        Some(x) => x.inner_html(),
                        None => {
                            error!("性感写真分类没有搜索到h3标签");
                            return false;
                        }
                    };
                    h3_str.contains("性感写真分类")
                })
                .unwrap();
            let parse_res = parse_main_page_section(series);
            self_tmp.xiezhen = parse_res;
        }

        //~ 人梯射影分类
        {
            let mut aside_series = main_page_doc.select(&aside_series_selector);
            let series = aside_series
                .find(|ele| {
                    let h3_str = match ele.select(&h3_selector).next() {
                        Some(x) => x.inner_html(),
                        None => {
                            error!("人体摄影分类没有搜索到h3标签");
                            return false;
                        }
                    };
                    h3_str.contains("人体摄影分类")
                })
                .unwrap();
            let parse_res = parse_main_page_section(series);
            self_tmp.renti_sheying = parse_res;
        }

        // //~ 承人影骗分类
        // {
        //     let mut aside_series = main_page_doc.select(&aside_series_selector);
        //     let series = aside_series
        //         .find(|ele| {
        //             let h3_str = match ele.select(&h3_selector).next() {
        //                 Some(x) => x.inner_html(),
        //                 None => {
        //                     error!("成人影片分类有搜索到h3标签");
        //                     return false;
        //                 }
        //             };
        //             h3_str.contains("成人影片分类")
        //         })
        //         .unwrap();
        //     let parse_res = parse_main_page_section(series);
        //     self_tmp.chengren_yingpian = parse_res;
        // }

        //~ 晓说分类
        // {
        //     let mut aside_series = main_page_doc.select(&aside_series_selector);
        //     let series = aside_series
        //         .find(|ele| {
        //             let h3_str = match ele.select(&h3_selector).next() {
        //                 Some(x) => x.inner_html(),
        //                 None => {
        //                     error!("小说分类有搜索到h3标签");
        //                     return false;
        //                 }
        //             };
        //             h3_str.contains("小说分类")
        //         })
        //         .unwrap();
        //     let parse_res = parse_main_page_section(series);
        //     self_tmp.xiaoshuo = parse_res;
        // }

        self_tmp
    }

    /// 解析页面分页，获取该页面中的所有分页页码和对应的URL
    ///
    /// 比如，解析某个photo汇总页`/photos/series-5f1476781eab4.html`中的 **所有页码** 信息
    ///
    /// 调用该方法后，请求返回值中的每个页面，并对所请求页面返回的html值调用`parse_serie_page()`来获取每一页中的作品信息
    ///
    /// ```text
    /// <div class="pager">
    ///     <div><a class="prev">上一页</a>
    ///          <a href="/photos/series-5f1476781eab4/1.html current="true">1</a>"
    ///          <a href="/photos/series-5f1476781eab4/2.html">2</a>
    ///          <a href="/photos/series-5f1476781eab4/3.html">3</a> ...
    ///          <a href="/photos/series-5f1476781eab4/359.html">359</a>
    ///          <a href="/photos/series-5f1476781eab4/2.html" class="next">下一页</a>
    ///     </div>
    /// </div>
    /// ```
    ///
    /// 返回值格式，其中元组中的第二个布尔值元素表示该分页是否是当前正在解析的页：[
    ///   ("https://xchina.co/photos/series-5f1476781eab4/1.html", true),
    ///   ...,
    ///   ("https://xchina.co/photos/series-5f1476781eab4/359.html", false),
    /// ]
    pub async fn parse_pages_urls(&self, url: &str) -> Vec<(String, bool)> {
        let mut urls = vec![];

        let html_str = match self.get_html(url).await {
            Some(s) => s,
            None => return urls,
        };

        let doc = Html::parse_document(&html_str);

        let this_page_url = url.to_string();
        // 提取该页的前缀：https://xchina.co
        let url_origin = {
            let o = Url::parse(&this_page_url).unwrap().origin();
            o.ascii_serialization()
        };

        // 提取所有的page数量和各页的URL
        let pager_selector = Selector::parse("div.pager div a").unwrap();

        let mut page_infos = HashMap::new();
        let mut current_page_num = 0;
        for ele in doc.select(&pager_selector) {
            let v = ele.value();
            let url = v.attr("href");
            let current = v.attr("current");
            let page_num = match ele.inner_html().parse::<u16>() {
                Err(_) => continue,
                Ok(n) => n,
            };
            if current == Some("true") {
                current_page_num = page_num;
            }
            page_infos.insert(url.unwrap(), page_num);
        }
        // 有的只有单页，没有页码。则只添加当前单页并返回
        if page_infos.is_empty() {
            error!("{} 没有其它页码", this_page_url);
            urls.push((this_page_url, true));
            return urls;
        }

        // 选出页码最大的URL和页码值
        let (max_page_url, max_page_num) = page_infos.iter().max_by(|a, b| a.1.cmp(b.1)).unwrap();

        // 把后缀去掉，提取url前面的公共部分，
        // 例如 /photos/series-5f1476781eab4/359.html 提取为 /photos/series-5f1476781eab4
        let (base_path, _) = max_page_url
            .rsplit_once('/')
            .expect(&format!("can't split page_url by '/': {}", max_page_url));

        // 合成所有的url
        if max_page_num >= &2 {
            for i in 1..=*max_page_num {
                let url = format!("{}/{}/{}.html", url_origin, base_path, i);
                urls.push((url, i == current_page_num));
            }
        }

        urls
    }

    /// 解析每个分类系列的页面，获取分类的所有作品列表(即该页中的作品列表)，以及每个作品对应的所有url页面
    /// 比如，解析某个photo汇总页`/photos/series-5f1476781eab4.html`中的所有作品列表信息
    pub async fn parse_serie_page(&self, url: &str) -> Vec<ContentInfo> {
        let mut contents = Vec::new();

        let html_str = match self.get_html(url).await {
            Some(s) => s,
            None => return contents,
        };

        let doc = Html::parse_document(&html_str);

        // 当前页中的作品列表格式：
        // <div class="list" cols="2">
        //     <div class="item">
        //         <a href="/photo/id-64c4abcd9026b.html" target="_blank">
        //         <img src="https://img.xchina.biz/photos/64c4abcd9026b/0001_600x0.jpg" alt="萌汉药baby"></a>
        //         <div>
        //             <div><a href="/photos/series-5f1476781eab4.html"><i class="fa fa-stop-circle"></i>&nbsp;秀仍网</a></div>
        //
        //             <div>   //////////////// 这是模特或演员，可能不存在`<div>&nbsp;</div>`，可能是多个人的合集
        //                 <div class="actorsOrModels"><a href="/model/id-5f5b7d2aca64c.html" target="_blank">萌汉药</a></div>
        //             </div>
        //
        //         </div>
        //         <div><a href="/photo/id-64c4abcd9026b.html" target="_blank">萌汉药baby</a></div>
        //         <div><div><i class="fa fa-clock-o"></i>&nbsp;2023-07-18</div></div>
        //         <div class="tag">
        //             <div>60P</div>     ///////////// 这个作品中的数量，如果带视频，格式：<div>74P + 1V</div>
        //             <div empty="true"></div>
        //             <div empty="true"></div>
        //         </div>
        //     </div>
        // </div>
        let item_selector = Selector::parse("div.list div.item").unwrap();

        // 找到所有作品列表，每个作品是一个item
        let items = doc.select(&item_selector);
        for item in items {
            let content = parse_content_from_series(url, item);
            if content.is_none() {
                continue;
            }
            debug!("{:#?}", content);
            contents.push(content.unwrap());
        }

        contents
    }

    /// 解析单个内容页面，获取该页面中所有图片和视频的url。该方法已经将获取到的图片url存入content中
    ///
    /// 例如，解析`https://xchina.co/photo/id-64c4abcd9026b.html`页
    ///
    /// 作品页面信息：
    /// ```text
    /// <div class="tab-contents">
    ///     <div class="tab-content video-info" id="tab_1" style="display: block;">
    ///         <div><i class="fa fa-address-card-o"></i>萌汉药baby</div>  ///// 标题
    ///         <div><i class="fa fa-picture-o"></i>60P</div>     /////// 作品内容数量，可能带V，`60P + 3V`
    ///         <div><i class="fa fa-video-camera"></i>
    ///             <a href="/photos/series-63959b9c87149.html">秀人网旗下</a>
    ///             <span class="joiner">-</span>
    ///             <a href="/photos/series-5f1476781eab4.html">秀人网</a>   /////// 作品分类
    ///         </div>
    ///         <div><i class="fa fa-calendar"></i>2023-07-18</div>     ////// 作品发布日期，可能没有发布日期
    ///         <div><i class="fa fa-female"></i>
    ///             <div class="actorsOrModels">        /////// 演员或模特，有的没有模特
    ///                 <a href="/model/id-5f5b7d2aca64c.html" target="_blank">萌汉药</a>
    ///             </div>
    ///         </div>
    ///         <div><i class="fa fa-tags"></i>
    ///             <div class="contentTag">丝袜</div>&nbsp;<i role="button" action="tag"
    ///                 style="cursor: pointer;" class="fa fa-question-circle" title="标签说明"></i>
    ///         </div>
    ///     </div>
    /// </div>
    /// ```
    ///
    /// 视频列表从html页面中正则匹配得到这样的行，然后serde_json反序列化，
    /// 没有视频的页面，不存在这一行
    /// ```text
    /// var videos = [{ "url": "\/photos\/64c4cfb6d472f\/0003.mp4", "filename": "0003.mp4", "filesize": "29M" }, { "url": "\/photos\/64c4cfb6d472f\/0001.mp4", "filename": "0001.mp4", "filesize": "8M" }, { "url": "\/photos\/64c4cfb6d472f\/0002.mp4", "filename": "0002.mp4", "filesize": "15M" }];
    /// ```
    ///
    /// 图片url
    /// ```text
    /// <div class="article mask">
    ///     <div class="photos">
    ///         <a href="/photoShow.php?server=1&amp;id=64c4cfb6d472f&amp;index=0&amp;pageSize=18">
    ///         <figure class="item"
    ///             style="padding-bottom: 57.5%; background-image: url('https://img.xchina.biz/photos/64c4cfb6d472f/0001_600x0.jpg');">
    ///             <img class="cr_only"
    ///                 src="https://img.xchina.biz/photos/64c4cfb6d472f/0001_600x0.jpg"
    ///                 alt="喵吉《浣溪沙·端午》 (1/98)">
    ///             <div class="tag"><div>No. 1</div></div>
    ///         </figure>
    ///         <a href="/photoShow.php?server=1&amp;id=64c4cfb6d472f&amp;index=1&amp;pageSize=18">
    ///             <figure class="item"
    ///                 style="padding-bottom: 141.5%; background-image: url('https://img.xchina.biz/photos/64c4cfb6d472f/0002_600x0.jpg');">
    ///                 <img class="cr_only"
    ///                     src="https://img.xchina.biz/photos/64c4cfb6d472f/0002_600x0.jpg"
    ///                     alt="【国模人体】喵小吉《浣溪沙·端午》 (2/98)">
    ///                 <div class="tag">
    ///                     <div>No. 2</div>
    ///                 </div>
    ///             </figure>
    ///         </a>
    ///     </div>
    /// </div>
    /// ```
    pub async fn content_urls_one_page(&self, url: &str) -> Option<Content> {
        let html_str = match self.get_html(url).await {
            Some(s) => s,
            None => return None,
        };

        parse_content_urls_in_page(url, &html_str)
    }
}

impl PageParser {
    /// 给定一个分类url，解析分页，并获取所有分页中的作品信息。
    ///
    /// 注意，有些分类中，有非常多的分页，几百页甚至接近上千页，因此并发多任务解析，并且通过通道来发送已经解析的页面
    ///
    /// 例如，给定如此url: https://xchina.co/photos/series-5f1476781eab4/1.html
    pub async fn parse_multi_serie_pages(&self, serie_urls: Vec<String>) -> Vec<ContentInfo> {
        let content_infos = Arc::new(RwLock::new(Vec::new()));

        // 50个任务并发解析各分页中的内容
        let semaphore = Arc::new(Semaphore::new(50));
        let mut tasks = vec![];
        for url in serie_urls {
            // let sp_client = self.splash_client.clone();
            let c_self = self.clone();
            let sem = semaphore.clone();
            let c_infos = content_infos.clone();
            let task = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                let infos = c_self.parse_serie_page(&url).await;
                c_infos.write().await.extend(infos);
            });
            tasks.push(task);
        }

        for task in tasks {
            let _ = task.await;
        }

        Arc::try_unwrap(content_infos).unwrap().into_inner()
    }

    /// 给定一个作品url，解析分页，并获取所有分页中的视频和图片url信息。返回None，表示无法解析成功
    ///
    /// 注意，有的作品有很多很多页，因此并发多任务解析
    ///
    /// 例如，给定如此url: https://xchina.co/photo/id-64c4abcd9026b/1.html
    pub async fn all_content_urls(&self, content_url: &str) -> Option<Content> {
        let contents = Arc::new(RwLock::new(Vec::new()));

        // 请求给定的url，得到html字符串
        let html_str = self.get_html(content_url).await?;

        // 获取到该作品的所有分页url
        let page_urls = self.parse_pages_urls(&content_url).await;
        debug!("获得所有页码: {:#?}", page_urls);

        // 解析当前页中的图片和视频.
        // 为None表示当前页没有解析到图片，可能是超出有效页范围的页码，可能是其它非作品页
        // 如果为None，则后面应该解析current=true的页
        let mut parse_current_flag = false;
        match parse_content_urls_in_page(content_url, &html_str) {
            Some(c) => contents.write().await.push(c),
            None => parse_current_flag = true,
        }

        // 20个任务并发解析各分页中的内容
        let semaphore = Arc::new(Semaphore::new(20));
        let mut tasks = vec![];

        for (page_url, is_current_page) in page_urls {
            let sem = semaphore.clone();
            let contents = contents.clone();
            let c_self = self.clone();
            // let splash_client = splash_client.clone();
            let task = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                // 是当前解析的页面，如果前面解析出了当前页面，则无需再次重复解析，否则应该解析该页
                if is_current_page && !parse_current_flag {
                    return;
                }

                if let Some(c) = c_self.content_urls_one_page(&page_url).await {
                    contents.write().await.push(c);
                }
            });
            tasks.push(task);
        }

        for task in tasks {
            let _ = task.await;
        }

        let mut contents = Arc::try_unwrap(contents).unwrap().into_inner();
        match contents.len() {
            0 => return None,
            1 => return Some(contents.remove(0)),
            _ => {
                let mut content = contents[0].clone();
                for c in contents.drain(1..) {
                    content.img_urls.extend(c.img_urls);
                }
                Some(content)
            }
        }
    }
}

/// 解析每个页面对应的url。返回None，表示找不到页面的url。
///
/// 从页面文档的head标签中解析`og:url`
///
/// ```text
/// <head>
///     <meta name="twitter:title" content="Vol. 7092 萌汉药baby - 秀仍网">
///     <meta name="twitter:image" content="https://img.xchina.biz/photos/64c4abcd9026b/0001.jpg">
///     <meta property="og:url" content="https://xchina.co/photo/id-64c4abcd9026b.html">
///     <meta property="og:title" content="Vol. 7092 萌汉药baby - 秀仍网">
/// </head>
/// ```
#[allow(dead_code)]
fn parse_page_url(doc: &Html) -> Option<String> {
    let selector = Selector::parse("head meta").unwrap();
    let elems = doc.select(&selector);
    for elem in elems {
        let value = elem.value();
        if value.attr("property") == Some("og:url") {
            return value.attr("content").map(|x| x.to_string());
        }
    }
    None
}

/// 解析主页中的每个分类系列，并返回系列中的列表信息，包含名称、url、数量
///
/// 参数为系列节点.
///
/// 系列格式：
/// ```text
/// <div class="series">
///   <h3>写针分类</h3>
///   <a href="/photos/kind-1.html">
///       <div>全部写针</div>
///   </a>
///   <a href="/photos/series-5f14806585bef.html">
///       <div>头条女神 (53)</div>
///   </a>
/// </div>
/// ```
fn parse_main_page_section(section: ElementRef) -> HashMap<String, (String, u16)> {
    let mut res = HashMap::new();

    let section_href_selector = Selector::parse("a").unwrap();
    let section_names_selector = Selector::parse("div").unwrap();
    let section_infos = section.select(&section_href_selector);
    // <a href="/photos/series-63959b9c87149.html">
    //     <div>秀仍网旗下 (10607)</div>
    // </a>
    // <a href="/photos/series-5f1476781eab4.html">
    //     <div class="sub">秀仍网 (6820)</div>
    // </a>
    for info in section_infos {
        // 分类的链接
        let section_href = info.value().attr("href").unwrap().to_string();

        // 分类的名称信息(`秀仍网 (6820)`、`Pure Media (79)`)
        let section_name = info
            .select(&section_names_selector)
            .next()
            .unwrap()
            .inner_html();
        // 把名称和数字提取出来
        let mut split = section_name.split(&['(', ')']).filter(|x| !x.is_empty());
        // 名称
        let section_name = split.next().unwrap().trim_end();
        // 名称含有全部的跳过
        if section_name.contains("全部") {
            continue;
        }
        // 数量(没有数字的，跳过，它可能是汇总页面)
        let section_count = match split.next() {
            None => continue,
            Some(n) => match n.parse::<u16>() {
                Ok(x) => x,
                Err(_e) => {
                    error!("`{}` 中的 `{}`无法解析为数值", section_name, n);
                    continue;
                }
            },
        };

        res.insert(section_name.to_string(), (section_href, section_count));
    }

    res
}

/// 解析作品列表页面的一个item，每一个item对应每一个作品展示卡片
/// ```text
/// <div class="item">
///     <a href="/photo/id-64c4abcd9026b.html" target="_blank">
///     <img src="https://img.xchina.biz/photos/64c4abcd9026b/0001_600x0.jpg" alt="萌汉药baby"></a>
///     <div>
///         <div><a href="/photos/series-5f1476781eab4.html"><i class="fa fa-stop-circle"></i>&nbsp;秀仍网</a></div>
///         <div>   //////////////// 这是模特或演员，可能不存在`<div>&nbsp;</div>`，可能是多个人的合集
///             <div class="actorsOrModels"><a href="/model/id-5f5b7d2aca64c.html" target="_blank">萌汉药</a></div>
///         </div>
///     </div>
///     <div><a href="/photo/id-64c4abcd9026b.html" target="_blank">萌汉药baby</a></div>
///     <div><div><i class="fa fa-clock-o"></i>&nbsp;2023-07-18</div></div>
///     <div class="tag">
///         <div>60P</div>     ///////////// 这个作品中的数量，如果带视频，格式：<div>74P + 1V</div>
///         <div empty="true"></div>
///         <div empty="true"></div>
///     </div>
/// </div>
/// ```
fn parse_content_from_series(this_page_url: &str, item: ElementRef) -> Option<ContentInfo> {
    // 获取该作品的页面url以及该页面所展示图片的img url(展示的img url取自作品内容之一)
    // 例如 'https://img.xchina.biz/photos/64c4abcd9026b/0001_600x0.jpg'
    // 有些无法解析出img url，可能是广告，跳过
    //
    // <a href="/photo/id-64c4abcd9026b.html" target="_blank">  ////// page url
    //   <img src="https://img.xchina.biz/photos/64c4abcd9026b/0001_600x0.jpg" alt="萌汉药baby">
    // </a>
    let (item_title, show_url, page_url) = {
        // img标签包含base url，img标签的父标签<a href>包含页面url
        let item_url_selector = Selector::parse("div.item img").unwrap();
        let img_url_elem = item.select(&item_url_selector).next();
        if img_url_elem.is_none() {
            return None;
        }

        let img_tag = img_url_elem.unwrap();
        let img_url = img_tag.value().attr("src").unwrap();
        let item_title = img_tag
            .value()
            .attr("alt")
            .unwrap_or_else(|| "无标题")
            .trim();

        let page_url = {
            let page_url_tag = ElementRef::wrap(img_tag.parent().unwrap()).unwrap();
            let page_url = page_url_tag.value().attr("href").unwrap();
            let url = Url::parse(this_page_url).unwrap();
            let url_prefix = url.origin().ascii_serialization();
            format!("{}{}", url_prefix, page_url)
        };

        (item_title, img_url, page_url)
    };

    // 获取作品中的内容数量，如果带视频，格式：<div>74P + 1V</div>
    //   <div class="tag">
    //       <div>60P</div>
    //       <div empty="true"></div>
    //       <div empty="true"></div>
    //   </div>
    let (jpg_count, video_count) = {
        let item_num_selector = Selector::parse("div.item div.tag div").unwrap();
        let ele = item.select(&item_num_selector);
        let texts = ele.map(|t| t.text()).flatten().collect::<Vec<_>>();
        // 找到带字母`P`的字符串，它可能格式`60P`，可能是`60P + 3V`
        let text = texts.into_iter().find(|x| x.contains("P"));
        if text.is_none() {
            error!("无法从{}获取作品数量信息", this_page_url);
            return None;
        }
        let mut n = text
            .unwrap()
            .split(&['P', 'V', '+', ' '])
            .filter(|x| !x.is_empty())
            .map(|x| x.parse::<u16>().expect(&format!("parse u16 {} failed", x)));

        (n.next().unwrap(), n.next().unwrap_or_default())
    };

    // 获取该作品的所属分类，例如，分类为"秀仍网"，有的没有分类，默认设置为"未分类"
    // <a href="/photos/series-5f1476781eab4.html"><i class="fa fa-stop-circle"></i>&nbsp;秀仍网</a>
    let fen_lei = {
        let item_fenlei_selector = Selector::parse("div.item i.fa-stop-circle").unwrap();
        match item.select(&item_fenlei_selector).next() {
            None => "无分类".to_string(),
            Some(elem) => {
                let parent_tag = ElementRef::wrap(elem.parent().unwrap()).unwrap();
                let fenlei = parent_tag.text().collect::<Vec<&str>>().join("");
                fenlei.trim().to_string()
            }
        }
    };

    // 获取actor或Model
    // <div>
    //    <div class="actorsOrModels"><a href="/model/id-5f5b7d2aca64c.html" target="_blank">萌汉药</a></div>
    // </div>
    //
    // 可能不存在，则`<div>&nbsp;</div>`
    //
    // 可能是两个或多个人的合集，格式如下：下面的格式在页面中显示为 `鱼紫酱/杏子Yada`
    // <div class="actorsOrModels">
    //     <a href="/model/id-5fb4f11aec363.html" target="_blank">鱼紫酱</a>
    //     <span class="delimiter">/</span>
    //     <a href="/model/id-64ac062f54fd5.html" target="_blank">杏子Yada</a>
    // </div>
    let actor = {
        let item_actor_selector = Selector::parse("div.item div.actorsOrModels a").unwrap();
        let act = item.select(&item_actor_selector);
        let name = act.map(|name| name.text()).flatten().collect::<Vec<_>>();
        if name.is_empty() {
            "无名".to_string()
        } else {
            name.join("-").trim().to_string()
        }
    };

    // 获取作品发布日期，有的作品没有日期，默认都设置为1970-01-01
    // <div><div><i class="fa fa-clock-o"></i>&nbsp;2023-07-18</div></div>
    let pub_date = {
        let item_date_selector = Selector::parse("div.item i.fa-clock-o").unwrap();
        match item.select(&item_date_selector).next() {
            None => "1970-01-01".to_string(),
            Some(ele) => {
                let parent_tag = ElementRef::wrap(ele.parent().unwrap()).unwrap();
                let date = parent_tag.text().collect::<Vec<&str>>().join("");
                date.trim().to_string()
            }
        }
    };

    // println!(
    //     "---------- 发布日期: {}, url: {}, 分类: {}, 名字：{}, 图片数量: {}, 视频数量: {}",
    //     pub_date, base_url, fen_lei, actor, jpg_count, video_count
    // );

    let content_info = ContentInfo {
        title: item_title.to_string(),
        actor,
        fen_lei,
        pub_date,
        show_url: show_url.to_string(),
        jpg_count,
        video_count,
        page_url,
    };

    Some(content_info)
}

fn parse_content_urls_in_page(this_page_url: &str, html_str: &str) -> Option<Content> {
    let doc = Html::parse_document(&html_str);

    // 获取作品内容的img_url，img_url从head标签的"og:image"获取并截取
    // <head>
    //     <meta property="og:image" content="https://img.xchina.biz/photos/64c4abcd9026b/0001.jpg">
    // </head>
    let img_url = {
        let selector = Selector::parse("head meta").unwrap();
        let elems = doc.select(&selector);
        let mut img_url = None;
        for elem in elems {
            let value = elem.value();
            if value.attr("property") == Some("og:image") {
                img_url = value.attr("content").map(|x| x.to_string());
                break;
            }
        }
        img_url
    };

    // 获取作品基本信息，包括作品内容的数量、作品发布日期、作品参演演员、作品分类等
    let content_info = {
        let content_info_slct = Selector::parse("div.tab-contents div.tab-content").unwrap();
        let content_info = match doc.select(&content_info_slct).next() {
            Some(c) => c,
            None => {
                error!("({}) 不是作品页, html_str: {}", this_page_url, html_str);
                return None;
            }
        };

        // 返回的Content是没有设置show_url的
        let mut content_info = match parse_content_info(&this_page_url, content_info) {
            None => return None,
            Some(c) => c,
        };
        if let Some(img_url) = img_url {
            content_info.show_url = img_url;
        }
        content_info
    };

    // 获取作品页面中的所有视频url，视频列表可能是空列表
    let videos = parse_content_videos(&content_info, &doc);

    // 获取作品页面中的所有图片url，图片url列表可能是空列表
    let jpgs = parse_content_imgs(&doc);
    if jpgs.is_empty() {
        error!("地址页 `{}` 没有图片内容", content_info.page_url);
        return None;
    }

    let content = Content {
        info: content_info,
        img_urls: jpgs,
        videos,
    };

    debug!("解析作品分页得到内容: {:#?}", content);
    Some(content)
}

/// 解析作品详情页的作品信息
/// 作品页面信息：
/// ```text
/// <div class="tab-content video-info" id="tab_1" style="display: block;">
///     <div><i class="fa fa-address-card-o"></i>萌汉药baby</div>     /// 作品标题
///     <div><i class="fa fa-picture-o"></i>60P</div>     /////// 作品内容数量，可能带V，`60P + 3V`
///     <div><i class="fa fa-video-camera"></i>
///         <a href="/photos/series-63959b9c87149.html">秀仍网旗下</a>
///         <span class="joiner">-</span>
///         <a href="/photos/series-5f1476781eab4.html">秀仍网</a>   /////// 作品子分类
///     </div>
///     <div><i class="fa fa-calendar"></i>2023-07-18</div>     ////// 作品发布日期，可能没有发布日期
///     <div><i class="fa fa-female"></i>
///         <div class="actorsOrModels">        /////// 演员或模特，有的没有模特
///             <a href="/model/id-5f5b7d2aca64c.html" target="_blank">萌汉药</a>
///         </div>
///     </div>
///     <div><i class="fa fa-tags"></i>
///         <div class="contentTag">丝袜</div>&nbsp;<i role="button" action="tag"
///             style="cursor: pointer;" class="fa fa-question-circle" title="标签说明"></i>
///     </div>
/// </div>
/// ```
fn parse_content_info(this_page_url: &str, content_item: ElementRef) -> Option<ContentInfo> {
    // 获取作品标题
    let title = {
        let title_selector = Selector::parse("div.tab-content i.fa-address-card-o").unwrap();
        match content_item.select(&title_selector).next() {
            None => "无标题".to_string(),
            Some(e) => match get_parent_text(e).first() {
                Some(t) => t.to_string(),
                None => "无标题".to_string(),
            },
        }
    };

    // 获取作品内容数量(类似：60P 或 60P + 3V)
    let (jpg_count, video_count) = {
        let content_count_selector = Selector::parse("div.tab-content i.fa-picture-o").unwrap();
        let (jpg_cnt, video_cnt) = match content_item.select(&content_count_selector).next() {
            None => {
                error!("无法从{}获取作品数量信息", this_page_url);
                return None;
            }
            // 从父元素中取得数量字符串
            Some(e) => match get_parent_text(e).first() {
                None => {
                    error!("无法从{}获取作品数量信息", this_page_url);
                    return None;
                }
                // 60P 或 60P + 3V
                Some(str) => {
                    let mut cnts = str
                        .split(&['P', 'V', '+', ' '])
                        .filter(|x| !x.is_empty())
                        .map(|x| x.parse::<u16>().expect(&format!("parse {} to u16", x)));
                    (cnts.next().unwrap(), cnts.next().unwrap_or_default())
                }
            },
        };
        (jpg_cnt, video_cnt)
    };

    // 获取作品分类。可能是父子分类(只获取子分类)，可能是独立分类.
    // 选择 i.fa-video-camera 标签后再得到父元素，从父元素获取所有文本，再取最后一个文本即可
    //
    // 单分类:
    // <div>
    //     <i class="fa fa-video-camera"></i>
    //     <a href="/photos/series-61b997728043b.html">尤美</a>
    // </div>
    //
    // 子分类:
    // <div><i class="fa fa-video-camera"></i>
    //     <a href="/photos/series-63959b9c87149.html">秀人网旗下</a>
    //     <span class="joiner">-</span>
    //     <a href="/photos/series-5f1476781eab4.html">秀人网</a>   /////// 子分类
    // </div>
    let fen_lei = {
        let fenlei_selector = Selector::parse("div.tab-content i.fa-video-camera").unwrap();
        let fenlei_item = content_item.select(&fenlei_selector).next();
        if fenlei_item.is_none() {
            error!("无法获得 `{}` 中的分类", this_page_url);
            return None;
        }
        let texts_iter = get_parent_text(fenlei_item.unwrap()).into_iter();
        let fenlei = texts_iter.rev().find(|x| !x.is_empty());
        match fenlei {
            Some(x) => x,
            None => {
                error!("无法获得 `{}` 中的分类", this_page_url);
                return None;
            }
        }
    };

    // 获取作品发布日期，可能没有日期，没有日期默认都设置为'1970-01-01'
    // <div><i class="fa fa-calendar"></i>2023-07-18</div>
    let pub_date = {
        let date_selector = Selector::parse("div.tab-content i.fa-calendar").unwrap();
        let date_item = content_item.select(&date_selector).next();
        match date_item {
            None => "1970-01-01".to_string(),
            Some(e) => match get_parent_text(e).first() {
                Some(x) => x.to_string(),
                None => "1970-01-01".to_string(),
            },
        }
    };

    // 获取作品参与演员，有的没有演员，没有演员默认为"无名"
    // <div><i class="fa fa-female"></i>
    //     <div class="actorsOrModels">        ///////
    //         <a href="/model/id-5f5b7d2aca64c.html" target="_blank">萌汉药</a>
    //     </div>
    // </div>
    let actor = {
        let actor_selector = Selector::parse("div.tab-content div.actorsOrModels a").unwrap();
        let actor_item = content_item.select(&actor_selector).next();
        match actor_item {
            None => "无名".to_string(),
            Some(e) => e.inner_html().trim().to_string(),
        }
    };

    // debug!(
    //     "演员: {}, 图片数量: {}, 视频数量: {}, 所属分类: {}, 发布日期: {}",
    //     actor, jpg_count, video_count, fen_lei, pub_date
    // );

    Some(ContentInfo {
        actor,
        fen_lei,
        pub_date,
        page_url: this_page_url.to_string(),
        show_url: this_page_url.to_string(),
        jpg_count,
        video_count,
        title,
    })
}

/// 获取页面中的视频列表。视频列表从html doc中匹配得到这样的行，然后serde_json反序列化，没有视频的页面，不存在这一行。
/// ```text
/// <div class="main"><div>
///     <script>
///         var domain = "https://img.xchina.biz";
///         var videos = [{ "url": "\/photos\/64c4cfb6d472f\/0003.mp4", "filename": "0003.mp4", "filesize": "29M" }, { "url": "\/photos\/64c4cfb6d472f\/0001.mp4", "filename": "0001.mp4", "filesize": "8M" }, { "url": "\/photos\/64c4cfb6d472f\/0002.mp4", "filename": "0002.mp4", "filesize": "15M" }];
///     </script>
/// </div></div>
/// ```
///
/// 返回：
/// ```text
/// [Video { url: "https://img.xchina.biz/photos/64c4cfb6d472f/0003.mp4", filename: "0003.mp4", filesize: "29M" }, Video { url: "https://img.xchina.biz/photos/64c4cfb6d472f/0001.mp4", filename: "0001.mp4", filesize: "8M" }, Video { url: "https://img.xchina.biz/photos/64c4cfb6d472f/0002.mp4", filename: "0002.mp4", filesize: "15M" }]
/// ```
/// 也可能返回空列表
fn parse_content_videos(content_info: &ContentInfo, doc: &Html) -> Vec<Video> {
    let mut videos = Vec::new();

    // 视频数量为0，直接返回None
    if content_info.video_count == 0 {
        return videos;
    }

    let video_selector = Selector::parse("body div.main script").unwrap();
    let video_elem = doc.select(&video_selector);
    let mut domain_line = None;
    let mut videos_line = None;

    for elem in video_elem {
        for line in elem.inner_html().lines() {
            if line.contains("var domain") {
                domain_line = Some(line.to_string());
                debug!("domain_line: {:?}", domain_line);
                continue;
            }
            if line.contains("var videos") {
                videos_line = Some(line.to_string());
                debug!("videos_line: {:?}", videos_line);
                break;
            }
        }
    }

    // domain_line: "https://img.xchina.biz"
    if let Some(line) = domain_line {
        let s = line.replace(&[' ', ';', '"'], "");
        domain_line = s.split_once('=').map(|x| x.1.to_string());
    }
    // videos_line: "[{\"url\":\"\\/photos\\/64c4cfb6d472f\\/0003.mp4\",\"filename\":\"0003.mp4\",\"filesize\":\"29M\"}]"
    if let Some(line) = videos_line {
        let s = line.replace(&[' ', ';'], "");
        videos_line = s.split_once('=').map(|x| x.1.to_string());
    }

    // 如果都有值，反序列化videos_str，然后使用base_url补齐视频的完整url
    if let (Some(base_url), Some(videos_str)) = (domain_line, videos_line) {
        match serde_json::from_str::<Vec<Video>>(&videos_str) {
            Err(e) => {
                error!("反序列化失败({}): {}", e, videos_str);
            }
            Ok(mut vs) => {
                vs.iter_mut()
                    .for_each(|x| x.url = format!("{}{}", base_url, x.url));
                videos = vs;
            }
        };
    }
    videos
}

/// 获取页面中的图片的url列表。
/// ```text
/// <div class="article mask">
///     <div class="photos">
///         <a href="/photoShow.php?server=1&amp;id=64c4cfb6d472f&amp;index=0&amp;pageSize=18">
///         <figure class="item"
///             style="padding-bottom: 57.5%; background-image: url('https://img.xchina.biz/photos/64c4cfb6d472f/0001_600x0.jpg');">
///             <img class="cr_only"
///                 src="https://img.xchina.biz/photos/64c4cfb6d472f/0001_600x0.jpg"
///                 alt="喵吉《浣溪沙·端午》 (1/98)">
///             <div class="tag"><div>No. 1</div></div>
///         </figure>
///         <a href="/photoShow.php?server=1&amp;id=64c4cfb6d472f&amp;index=1&amp;pageSize=18">
///             <figure class="item"
///                 style="padding-bottom: 141.5%; background-image: url('https://img.xchina.biz/photos/64c4cfb6d472f/0002_600x0.jpg');">
///                 <img class="cr_only"
///                     src="https://img.xchina.biz/photos/64c4cfb6d472f/0002_600x0.jpg"
///                     alt="【国模人体】喵小吉《浣溪沙·端午》 (2/98)">
///                 <div class="tag">
///                     <div>No. 2</div>
///                 </div>
///             </figure>
///         </a>
///     </div>
/// </div>
/// ```
fn parse_content_imgs(doc: &Html) -> Vec<String> {
    let mut urls = Vec::new();
    let img_selector = Selector::parse("div.article div.photos figure.item img.cr_only").unwrap();

    // 有几种格式的图片url，如果文件名以600x0结尾，可去除这部分，获取更高分辨率的图片
    // https://img.xchina.biz/photos/64c4cfb6d472f/0001_600x0.jpg
    // https://img.xchina.biz/photos/5f55202b3e808/1080P_4000K_285318102.mp4_20200906_204731321_600x0.jpg
    // https://img.xchina.biz/photos/5f55202b3e808/_Cover_600x0.jpg
    // https://img.xchina.biz/photos/5f5681e7128bb/153132qsuubostfcuf3cf1.jpg
    for jpg_elem in doc.select(&img_selector) {
        let img_url = jpg_elem.value().attr("src").unwrap();
        match img_url.rsplit_once('_') {
            None => urls.push(img_url.to_string()),
            Some((left, right)) => {
                let ext = right.rsplit_once('.').unwrap().1;
                urls.push(format!("{}.{}", left, ext));
            }
        }
    }

    urls
}

/// 给定一个标签元素，返回父元素的文本。返回结果中已经过滤为空的字符串以及修剪前后缀空白
///
/// 例如，给定如下标签中的i标签元素，返回父元素div的文本，即60P
/// ```text
/// <div><i class="fa fa-picture-o"></i>60P</div>
/// ```
fn get_parent_text(elem: ElementRef) -> Vec<String> {
    let parent = ElementRef::wrap(elem.parent().unwrap()).unwrap();
    parent
        .text()
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect::<Vec<String>>()
}
