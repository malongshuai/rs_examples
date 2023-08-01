use number_range::NumberRangeOptions;
use time::macros::{format_description, offset};
use tracing_subscriber::{fmt::time::OffsetTime, EnvFilter};

pub fn enable_log() {
    let local_time_fmt =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]");
    let local_timer = OffsetTime::new(offset!(+8), local_time_fmt);

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "INFO");
    }
    let mut log_builder = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_timer(local_timer)
        .with_target(false);

    let max_level = EnvFilter::from_default_env().max_level_hint().unwrap();
    // 比Debug更详细时，开启更多日志记录
    if max_level >= tracing::Level::DEBUG {
        log_builder = log_builder
            .with_file(true)
            .with_line_number(true)
            .with_ansi(true);
    } else {
        log_builder = log_builder.with_level(false);
    }

    log_builder.init();
}

/// 解析范围字符串。规则(假设 given_num 参数给定值为15)：
///
/// - 离散值: `1,3,5,7,9`，表示第1 3 5 7 9
///
/// - 绝对范围值: `1~10,20~25`，表示第1到第10，第20到第25
///
/// - 相对范围值: `+10`，表示以given_num参数给定的数值为基准，向后再取10位，即15~25，总共11项
///
/// - 相对范围值: `-10`，表示以given_num参数给定的数值为基准，向前再取10位(最小到1)，即5~15，总共11项
///
/// 例如，假设given_num为9，该选项指定为`3,5,4~6,3~8,+5,-10`，其结果等价于`1~14`
pub fn parse_number_range(range_str: &str, given_num: i16) -> Vec<u16> {
    // 因使用`number_range` crate，不支持相对范围，因此，先将相对范围值移除，手动解析相对范围值，
    // 再转换为number_range支持的范围，最后通过number_range解析，最后排序、去重

    if given_num <= 0 {
        panic!("解析范围字符串时，传递了错误的值: {}", given_num);
    }

    // 先按逗号分隔为各个元素
    let split_str = range_str.split(',').into_iter();

    // 各个元素组合为一个Vec
    let mut split_arr = split_str
        .clone()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    // 取带`+`号的相对范围值
    let relative_range1 = split_str
        .clone()
        .filter(|x| x.starts_with('+'))
        .map(|x| x.parse::<i16>().expect("错误的范围值"))
        .collect::<Vec<i16>>();

    // 最多只允许有一个向后的相对范围
    if relative_range1.len() >= 2 {
        panic!("最多只能指定一个同方向(相对于当前页，向前还是先后)的相对范围");
    }

    // 如果存在向后的相对范围值，则先从split_arr中移除该字符串，
    // 然后手动解析为绝对范围值，再追加回split_arr中
    if let Some(i) = relative_range1.first() {
        split_arr.retain(|x| x != &format!("+{}", i));

        let abs_range = format!("{}~{}", given_num, given_num + i);
        split_arr.push(abs_range);
    }

    // 取带`-`号的相对范围值
    let relative_range2 = split_str
        .clone()
        .filter(|x| x.starts_with('-'))
        .map(|x| x.parse::<i16>().expect("错误的范围值"))
        .collect::<Vec<i16>>();

    // 最多只允许有一个向前的相对范围
    if relative_range2.len() >= 2 {
        panic!("最多只能指定一个同方向(相对于当前页，向前还是先后)的相对范围");
    }

    // 如果存在向前的相对范围值，则先从split_arr中移除该字符串，
    // 然后手动解析为绝对范围值，再追加回split_arr中，
    // 注意数值i是负数
    if let Some(i) = relative_range2.first() {
        split_arr.retain(|x| x != &i.to_string());

        let abs_range = format!("{}~{}", (given_num + i).max(1), given_num);
        split_arr.push(abs_range);
    }

    // 将split_arr重新使用逗号串联起来
    let range1_str = split_arr.join(",");

    // 交给number_range解析，并设置`~`为范围分隔符
    let mut rngs = NumberRangeOptions::<u16>::new()
        .with_range_sep('~')
        .parse(&range1_str)
        .expect(&format!("无法解析范围字符串: {}", range_str))
        .collect::<Vec<_>>();

    // 排序并去重
    rngs.sort();
    rngs.dedup();

    rngs
}

#[cfg(test)]
mod test {
    use super::parse_number_range;

    #[test]
    fn test_parse_range_str() {
        let range_str = "3,5,4~6,3~8,+5,-10";
        // println!("{:?}", parse_number_range(range_str, 9));
        assert_eq!(
            parse_number_range(range_str, 9),
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        );
    }
}
