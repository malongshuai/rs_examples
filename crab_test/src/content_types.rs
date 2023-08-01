//! 内容分类
//!

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

// 主页的各种分类
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct MainPageFenLei {
    /// 写针分类
    ///
    /// 各种写针分类的主汇总页面地址(比如Xiu人对应的汇总页面，You果对应的汇总页面)
    ///
    /// key为分类名称，Value为(Url, 分类的写针数量)
    ///
    /// 例如，key："秀仍网"，value: ("/photos/series-5f1476781eab4.html", 6820)
    /// ```
    ///  <a href="/photos/series-5f1476781eab4.html">
    ///     <div class="sub">秀仍网 (6820)</div>
    /// </a>
    /// ```
    pub xiezhen: HashMap<String, (String, u16)>,
    /// 人梯射影分类
    pub renti_sheying: HashMap<String, (String, u16)>,
    // /// 承人影骗分类
    // pub chengren_yingpian: HashMap<String, (String, u16)>,
    // /// 晓说分类
    // pub xiaoshuo: HashMap<String, (String, u16)>,
}

/// 作品信息(不包含作品中各内容的url)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentInfo {
    /// 作品分类，比如属于 秀仍网
    pub fen_lei: String,
    /// 作品参与演员或模特名称(例如: 猪猪)，没有名称的，默认为"无名"
    pub actor: String,
    /// 标题，没有标题的，默认为"无标题"
    pub title: String,
    /// 作品的发布日期，没有发布日期的，默认设置为"1970-01-01"
    pub pub_date: String,
    /// 页面的url，例如，
    /// https://xchina.co/photo/id-64c4abcd9026b.html
    /// https://xchina.co/photo/id-64c4abcd9026b/1.html
    pub page_url: String,
    /// 用于作品展示图片的img url，它来自于作品内容之一，
    /// 例如：'https://img.xchina.biz/photos/64c4abcd9026b/0001.jpg'，
    pub show_url: String,
    /// 作品中的图片数量
    pub jpg_count: u16,
    /// 作品中的视频数量
    pub video_count: u16,
}

impl ContentInfo {
    /// 返回该ContentInfo中作品内容的文件保存路径。
    ///
    /// 路径为给定save_dir目录下的子目录，子目录规则为：`分类/演员/标题_日期_<page_url>的后缀`，
    ///
    /// 例如，给定save_dir为/tmp，页面的url为`/photo/id-64c4abcd9026b.html`，
    /// 则对应的目录为：`/tmp/秀仍网/猪猪/标题_2023-05-05_id-64c4abcd9026b/`
    pub fn file_dir<T: AsRef<Path>>(&self, save_dir: T) -> PathBuf {
        let path = save_dir.as_ref().join(&self.fen_lei).join(&self.actor);

        // 变成https://xchina.co/photo/id-64c4abcd9026b 或 https://xchina.co/photo/id-64c4abcd9026b/1
        let no_suffix = self.page_url.strip_suffix(".html").unwrap();
        // 两种可能:
        //   - left: "https://xchina.co/photo", right: "id-64c4abcd9026b"
        //   - left: "https://xchina.co/photo/id-64c4abcd9026b", right: "1"
        let (left, right) = no_suffix.rsplit_once('/').unwrap();
        let id = if right.parse::<u16>().is_err() {
            right
        } else {
            left.rsplit_once('/').unwrap().1
        };

        let next_part = format!("{}_{}_{}", self.title, self.pub_date, id);

        path.join(next_part)
    }
}

/// 作品
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub info: ContentInfo,
    /// 所有图片URL，可能为空
    pub img_urls: Vec<String>,
    /// 所有视频URL，可能为空
    pub videos: Vec<Video>,
}

impl Content {
    /// 获取作品基本信息
    pub fn content_info(&self) -> &ContentInfo {
        &self.info
    }

    /// 该作品的所有url。
    ///
    /// 如果img_urls和videos都为空，则合成所有图片的url，但不保证合成的url是正确的。
    /// 合成方式：提取show_img_url的base url部分，然后从0001开始自增。
    /// 例如，根据`https://img.xchina.biz/photos/64c4abcd9026b/0001.jpg`，合成`0002.jpg 0003.jpg`等。
    ///
    /// 合成的url可能错误，所以应对合成的url进行一次尝试，如果尝试失败，则应请求作品页获得完整的正确的urls
    pub fn urls(&self) -> Vec<String> {
        let mut urls = Vec::new();

        // 如果不可合成，则返回空列表
        // 如果img_urls不为空，说明url已经填充了，直接返回，无需合成
        // 如果img_urls为空，但videos不为空，则直接返回，无需合成
        {
            if !self.can_merge_urls() {
                return urls;
            }

            if !self.img_urls.is_empty() {
                urls.extend(self.img_urls.clone());
                urls.extend(self.video_urls());
                return urls;
            }

            if !self.videos.is_empty() {
                return self.video_urls();
            }
        }

        // 只有img_urls和videos都为空时，才合成
        {
            let content_info = self.content_info();
            let base_url = self.base_url();
            let extension = self.extension();
            if content_info.jpg_count >= 1 {
                for i in 1..=content_info.jpg_count {
                    let url = format!("{}/{:04}.{}", base_url, i, extension);
                    urls.push(url);
                }
            }
            if content_info.video_count >= 1 {
                urls.extend(self.video_urls());
            }
        }

        urls
    }

    /// 从 show_img_url 截取该作品内容地址公共部分的 base_url
    pub fn base_url(&self) -> String {
        let show_url = &self.content_info().show_url;
        show_url.rsplit_once('/').unwrap().0.to_string()
    }

    /// 获取展示图片的后缀名，注意，后缀名中不包含前缀`.`
    /// 例如从`https://img.xchina.biz/photos/64c4abcd9026b/0001_600x0.jpg`获取的后缀为`jpg`
    pub fn extension(&self) -> String {
        let show_img_url = &self.content_info().show_url;
        show_img_url.rsplit_once('.').unwrap().1.to_string()
    }

    /// 视频的url
    pub fn video_urls(&self) -> Vec<String> {
        let base_url = self.base_url();
        let mut urls = Vec::new();
        for v in &self.videos {
            let url = format!("{}/{}", base_url, v.filename);
            urls.push(url);
        }
        urls
    }

    /// 该作品的内容的url列表能否通过 base url 进行合成。
    /// 如果show_img_url字段的filename部分的前部，是数值，则认为可以合成，否则不能合成，
    /// 例如`https://img.xchina.biz/photos/64c4abcd9026b/0001_600x0.jpg`的filename的前部是0001，认为可以合成，
    /// 而`https://img.xchina.biz/photos/5f5681e7128bb/152632jz3iritf303i3cja_600x0.jpg`的filename的前部是152632jz3iritf303i3cja，认为不可以合成，
    /// 但即便返回true，所合成的url也不保证是正确的。
    /// 如果要确保作品内容的url是正确的，应该解析作品页面
    pub fn can_merge_urls(&self) -> bool {
        let filename = self.content_info().show_url.rsplit_once('/').unwrap().1;
        filename
            .split(&['_', '.'])
            .next()
            .and_then(|x| x.parse::<u16>().ok())
            .is_some()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Video {
    pub url: String,
    pub filename: String,
    pub filesize: String,
}

#[allow(dead_code)]
#[cfg(test)]
mod test {
    use url::Url;

    use crate::content_types::Video;
    #[test]
    fn tt() {
        let str = r##"
        [{"url":"\/photos\/64c4cfb6d472f\/0003.mp4","filename":"0003.mp4","filesize":"29M"},{"url":"\/photos\/64c4cfb6d472f\/0001.mp4","filename":"0001.mp4","filesize":"8M"},{"url":"\/photos\/64c4cfb6d472f\/0002.mp4","filename":"0002.mp4","filesize":"15M"}]
        "##;

        let res = serde_json::from_str::<Vec<Video>>(str);
        println!("{:?}", res);
    }

    #[test]
    fn t() {
        let url1 = "https://xchina你.co/photos/series-5f1476781eab4.html";
        let url2 = "https://xchina.co/photos/series-5f1476781eab4/2.html";

        let url1 = Url::parse(url1).unwrap();
        let url2 = Url::parse(url2).unwrap();

        let o1 = url1.origin();
        let o2 = url2.origin();

        let s1 = o1.ascii_serialization();
        let s2 = o2.unicode_serialization();

        println!("url1 {:?}, url2: {:?}", s1, s2);
    }
}
