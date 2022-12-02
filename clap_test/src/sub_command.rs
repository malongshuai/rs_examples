use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
struct Opts {
    #[command(subcommand)]
    cmds: AddCommand,

    /// 给定api_key
    #[clap(long, short, env = "API_KEY")]
    api_key: String,

    /// 给定sec_key
    #[clap(long, short, env = "SEC_KEY")]
    sec_key: String,
}

#[derive(Debug, Subcommand)]
enum AddCommand {
    CancelOrder(CancelOrder),
    AssetTransfer(AssetTransfer),
}

/// 撤单操作
#[derive(Debug, Parser)]
struct CancelOrder {
    /// 指定symbol
    #[clap(short, long)]
    symbol: String,

    /// 指定order_id
    #[clap(short, long)]
    order_id: u64,
}

/// 资产划转
#[derive(Debug, Parser)]
struct AssetTransfer {
    /// 指定资产名称
    #[clap(short, long)]
    asset: String,

    /// 指定数量
    #[clap(short, long)]
    amount: f64,
}

fn main() {
    let opts = Opts::parse();
    println!("{:?}", opts);
}
