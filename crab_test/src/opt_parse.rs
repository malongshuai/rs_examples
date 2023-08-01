use crate::XCHAIN_BASE_URL;
use clap::{ArgGroup, Parser, Subcommand};
use std::{env, path::PathBuf, str::FromStr};
use url::Url;

/// 解析/下载 ×chinα.co 美图/视频
///
/// 不支持解析/下载视频页的视频，只支持解析/下载图片页中附带的视频
///
/// 可在程序所在目录或当前所在目录中创建 `.env` 文件来设置各选项的环境变量
#[derive(Debug, Parser)]
pub struct Opts {
    #[command(subcommand)]
    pub cmds: Cmds,

    /// Splash服务监听地址和端口，格式"http[s]://ip:port"，
    /// 也可以设置到环境变量 SPLASH_ADDR，
    ///
    /// 如果都没有设置，则默认"http://127.0.0.1:8050"
    #[clap(short, long, env = "SPLASH_ADDR")]
    pub splash_addr: Option<String>,

    /// 设置Splash请求时使用的Proxy，可以设置到环境变量 APP_PROXY
    ///
    /// 也可以使用系统默认的代理环境变量 http[s]_proxy
    ///
    /// 格式：<http[s]|socks5>://<IP>:<PORT>
    ///
    /// 例如，http://127.0.0.1:8118, socks5://127.0.0.1:1080
    ///
    /// 需注意，本程序以及Splash服务端都将使用该代理，因此，须确保Splash服务端能访问该代理地址
    #[clap(short, long, env = "APP_PROXY")]
    pub proxy: Option<String>,

    /// 指定下载目录，默认当前所在目录
    #[clap(short = 'o', long, env = "SAVE_DIR")]
    pub save_dir: Option<PathBuf>,

    /// 使用 debug 模式
    #[clap(long)]
    pub debug: bool,
}

#[derive(Debug, Subcommand)]
pub enum Cmds {
    // Parse(Parse),
    Parse(Parse),
    Download(Download),
    /// 无视该子命令，我用来调试功能的选项
    #[clap(subcommand, hide(true))]
    No,
}

/// 页面解析操作
#[derive(Debug, Parser)]
#[command(group(ArgGroup::new("only_one").required(false).args(["pages", "max_page"])))]
pub struct Parse {
    /// 指定要解析的页面url，可以是(参考以下示例)：
    ///
    /// - 主页: `https://xchina.co`，解析将得到各个分类(即侧边栏中的分类)信息  
    ///
    /// - 分类页: `https://xchina.co/photos/series-5f1476781eab4.html`，
    ///
    ///           通常包含字符"photos"或"model"，解析当前分类页中所有作品的信息
    ///
    /// - 作品页: `https://xchina.co/photo/id-64c4abcd9026b/1.html`，
    ///
    ///           通常包含字符"photo"，解析该作品页的信息以及作品的中所有url
    #[clap(short, long)]
    pub url: String,

    /// 解析/下载多个分类页(只适用于分类页). 和 --max-page 选项冲突
    ///
    /// 以下分类页的分类都是: https://xchina.co/photos/series-5f1476781eab4
    ///
    /// (1)."https://xchina.co/photos/series-5f1476781eab4.html"
    ///
    /// (2)."https://xchina.co/photos/series-5f1476781eab4/1.html"
    ///
    /// (3)."https://xchina.co/photos/series-5f1476781eab4/5.html"
    ///
    /// 其中(1)和(2)等价，都属于分类的第一页
    ///
    /// 假设url选项指定分类页"https://xchina.co/photos/series-5f1476781eab4/15.html"
    ///
    /// 以下几种参数值格式的含义：
    ///
    /// - 离散页: `1,3,5,7,9`，表示解析/下载该分类的第1 3 5 7 9页，url选项所给定的第15页不会被解析/下载
    ///
    /// - 绝对范围页: `1~10,20~25`，表示解析/下载第1到第10页，第20到第25页，url选项给定的第15页不会被解析/下载
    ///
    /// - 相对范围页: `+10`，表示解析/下载url选项给定的第15页以及其后10页，即15~25页，总共解析/下载11页
    ///
    /// - 相对范围页: `-10`，表示解析/下载url选项给定的第15页以及其前10页(最小到第1页)，即5~15页，总共解析/下载11页
    ///
    /// 例如，假设url选项给定的是第9页，该选项指定为`3,5,4~6,3~8,+5,-10`，其结果等价于`1~14`
    ///
    /// 如果不指定该选项，则默认仅解析/下载url选项所给定的页
    #[clap(short, long)]
    pub pages: Option<String>,

    /// 解析分类页所属分类的最大页码以及所有分页的页码url，而不是解析分类页中的作品信息
    ///
    /// 只有 --url 选项指定分类页时，该选项才有效
    ///
    /// 和 --pages 选项冲突
    #[clap(short, long)]
    pub max_page: bool,
}

/// 下载操作
#[derive(Debug, Parser)]
pub struct Download {
    /// 指定要下载的页面url，可以是(参考以下示例)：
    ///
    /// - 分类页: `https://xchina.co/photos/series-5f1476781eab4.html`，
    ///
    ///           通常包含字符"photos"或"model"，下载当前分类页中所有作品的信息
    ///
    /// - 作品页: `https://xchina.co/photo/id-64c4abcd9026b/1.html`，
    ///
    ///           通常包含字符"photo"，下载该作品页的信息以及作品的中所有url
    ///
    /// - 具体文件url：`https://img.xchina.biz/photos/64c4abcd9026b/0001.jpg`
    #[clap(short, long)]
    pub url: String,

    /// 下载多个分类页中的内容(只适用于分类页).
    ///
    /// 该选项参数的格式参考 parse 子命令的 `--pages` 选项的解释说明
    #[clap(short, long)]
    pub pages: Option<String>,

    /// 只接收三个值(不区分大小写)：a, v, p
    ///
    /// - v: 表示只下载视频文件  
    /// - p: 表示只下载图片文件  
    /// - a: 都下载(默认值)
    #[clap(short, long, default_value = "a")]
    pub only: DownloadType,
}

#[derive(Debug)]
pub struct SimleOpts {
    pub splash_addr: String,
    pub proxy: Option<String>,
    pub save_dir: PathBuf,
}

pub fn args_init() -> (SimleOpts, Opts) {
    let opts = Opts::parse();

    // 先检查子命令的选项是否合理
    match &opts.cmds {
        Cmds::Parse(c) => valid_parse_cmd(c),
        Cmds::Download(d) => valid_download_cmd(d),
        Cmds::No => {}
    }

    // 先尝试从程序所在目录读取.env文件，再尝试从当前目录读取.env文件
    let path = env::current_exe().unwrap().parent().unwrap().to_path_buf();
    if dotenvy::from_path(path).is_err() {
        if dotenvy::dotenv().is_err() {}
    };

    // 读取选项，再读环境变量，最后默认设置
    let splash_addr = opts
        .splash_addr
        .clone()
        .or_else(|| env::var("SPLASH_ADDR").ok())
        .or_else(|| Some("http://127.0.0.1:8050".to_string()))
        .unwrap();

    // 读取选项，再读环境变量，都没有则默认为空
    let proxy = opts.proxy.clone().or_else(|| env::var("APP_PROXY").ok());

    // 先读选项，再读环境变量，最后设置默认
    let save_dir = opts.save_dir.clone().unwrap_or_else(|| {
        env::var("SAVE_DIR").ok().map_or_else(
            || env::current_dir().unwrap(),
            |x| PathBuf::from_str(&x).unwrap(),
        )
    });

    if opts.debug {
        std::env::set_var("RUST_LOG", "info,crab_test=debug");
    }

    let simple_opts = SimleOpts {
        splash_addr,
        proxy,
        save_dir,
    };

    (simple_opts, opts)
}

/// 检查 parse 子命令的选项
fn valid_parse_cmd(cmd: &Parse) {
    let url_type = UrlType::parse(&cmd.url).expect(&format!("无效的url: {}", cmd.url));
    if !url_type.is_fenlei() {
        if cmd.max_page {
            panic!("指定 `--max-page` 选项时，`--url` 选项的参数必须是分类url")
        }

        if cmd.pages.is_some() {
            panic!("指定 `--pages` 选项时，`--url` 选项的参数必须是分类url")
        }
    }
}

/// 检查 download 子命令的选项
fn valid_download_cmd(cmd: &Download) {
    let url_type = UrlType::parse(&cmd.url).expect(&format!("无效的url: {}", cmd.url));
    if cmd.pages.is_some() && !url_type.is_fenlei() {
        panic!("指定 `--pages` 选项时，`--url` 选项的参数必须是分类url")
    }

    // if !["a", "v", "p"].contains(&cmd.only.to_lowercase().as_str()) {
    //     panic!("`--only` 选项的有效参数值为: [a, v, p]")
    // }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UrlType {
    /// 给定url是主页，固定值"https://xchina.co"
    MainPage(String),
    /// 给定url是分类页，分类页的url中含有"photos"或"model"
    ///
    /// 例如：
    ///
    /// - https://xchina.co/photos/series-5f1476781eab4.html
    ///
    /// - https://xchina.co/model/id-5fbe9c2dad0c3.html
    FenLei(String),

    /// 给定的url是作品页，作品页的url中含有"photo"
    ///
    /// 例如：https://xchina.co/photo/id-64c218099a02f.html
    ZuoPing(String),

    /// 给定的url是单个文件
    ///
    /// 例如：https://img.xchina.biz/photos/64c218099a02f/0001.jpg
    SingleFile(String),
}

impl UrlType {
    pub fn parse(url: &str) -> Option<Self> {
        let url1 = Url::parse(url).ok()?;

        // https://xchina.co 或 https://img.xchina.biz
        let prefix = url1.origin().ascii_serialization();

        // `/` 或 `/photo/id-64c218099a02f.html` 或 `/model/id-5fbe9c2dad0c3.html`
        let path = url1.path();

        // 如果prefix是https://xchina.co，且path是/，则是主页
        if prefix == XCHAIN_BASE_URL && path == "/" {
            return Some(Self::MainPage(prefix));
        }

        // 如果prefix是https://xchina.co，且path以"/photos/"或"/model"开头，则是分类页
        if prefix == XCHAIN_BASE_URL && (path.starts_with("/photos/") || path.starts_with("/model"))
        {
            return Some(Self::FenLei(url1.to_string()));
        }

        // 如果prefix是https://xchina.co，且path以"/photo/"开头，则是作品页
        if prefix == XCHAIN_BASE_URL && path.starts_with("/photo/") {
            return Some(Self::ZuoPing(url1.to_string()));
        }

        // 如果prefix不是https://xchina.co，且path的filename部分不是.html结尾的，则是单个文件
        let filename = path.split('/').filter(|x| !x.is_empty()).last().unwrap();
        if prefix != XCHAIN_BASE_URL && !path.ends_with(".html") && filename.contains(".") {
            return Some(Self::SingleFile(url1.to_string()));
        }

        None
    }

    pub fn url(&self) -> String {
        match self {
            UrlType::MainPage(x) => x.to_string(),
            UrlType::FenLei(x) => x.to_string(),
            UrlType::ZuoPing(x) => x.to_string(),
            UrlType::SingleFile(x) => x.to_string(),
        }
    }

    pub fn is_mainpage(&self) -> bool {
        matches!(self, Self::MainPage(_))
    }

    pub fn is_fenlei(&self) -> bool {
        matches!(self, Self::FenLei(_))
    }

    pub fn is_zuopin(&self) -> bool {
        matches!(self, Self::ZuoPing(_))
    }

    pub fn is_single_file(&self) -> bool {
        matches!(self, Self::SingleFile(_))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum DownloadType {
    #[default]
    All,
    Imgs,
    Videos,
}

impl FromStr for DownloadType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "a" => Ok(Self::All),
            "p" => Ok(Self::Imgs),
            "v" => Ok(Self::Videos),
            _ => Err("valid value is [a, p, v]"),
        }
    }
}
