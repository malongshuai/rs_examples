use std::str::FromStr;

use clap::{ArgGroup, Parser, ValueEnum};

#[derive(Debug, Clone)]
enum Gender {
    Male,
    Female,
}

impl FromStr for Gender {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "male" => Ok(Gender::Male),
            "female" => Ok(Gender::Female),
            _ => Err("invalid gender, valid: male/female"),
        }
    }
}

#[derive(Debug, ValueEnum, Clone)]
enum Vip {
    VIP0,
    VIP1,
    VIP2,
    VIP3,
}
impl Default for Vip {
    fn default() -> Self {
        Vip::VIP0
    }
}

/// represent a person
#[derive(Debug, Parser)]
// 参数组，指定pr和sp两个参数(如果存在这两个参数的话)不能同时使用
// #[clap(group(
//             ArgGroup::new("vers")
//                 .required(false)
//                 .args(&["pr", "sp"]),
//         ))]
struct Opts {
    // 必须设置该选项，支持短选项 -n 和长选项 --name
    /// the name
    #[clap(short, long)]
    name: String,

    // 不设置该选项时，默认值为20
    /// the age
    #[clap(short, long, default_value_t = 20)]
    age: u8,

    // 不设置该选项时，默认为None，是一个Enum，要求Gender实现ValueEnum，但这里使用了默认的try_from_str(需impl FromStr for Gender)
    // clap提供的enum验证值，参考vip字段，自定义try_from_str，参考...
    /// the gender
    #[clap(short, long)]
    gender: Option<Gender>,

    // 是一个Enum，要求Vip实现ValueEnum，
    // 这里提供了默认值，要求Vip实现Default(impl Default for Vip)，
    // 或者default_value = "vip1",
    // 对于value_enum的默认值，还可以设置default_value_t = Vip::VIP1,
    /// is a vip?
    #[clap(short, long, default_value_t)]
    #[arg(value_enum)]
    vip: Vip,

    // 布尔值，提供了该选项，则为true，省略该选项为false
    /// is a student?
    #[clap(short, long)]
    is_student: bool,

    // 首字母和gender字段相同，需修改short的名称，
    /// is good student?
    #[clap(short = 'G', long)]
    good_student: bool,

    // 如果没有设置该选项，则尝试从环境变量PHONE中读取，环境变量不存在则报错
    // env要求开启env features
    /// phone number
    #[clap(short, long, env)]
    phone: u64,

    // 没有指定如何解析的字段，被认为是非选项型参数，即位置参数
    /// position parameter
    friend: String,

    // Vec类型时，可接受0或多次选项及其参数的值，
    // 不指定选项时，vec为空，多次指定选项时，值被收集到vec，
    // 例如: --email 912@163.com --email 999@ya.com
    /// email，0 or 1+
    #[clap(long, env)]
    email: Vec<String>,
}

fn main() {
    let opts = Opts::parse();
    println!("{:?}", opts);
}
