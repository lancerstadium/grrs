use std::env::args;
use clap::Parser;

// 在文件中搜索模式并显示包含该模式的行。
#[derive(Parser)]
struct Cli {
    pattern: String,            // 要查找的字符串
    path: std::path::PathBuf,   // 要查看的文件
}

fn main() {
    let pattern = args().nth(1).expect("no pattern given");
    let path = args().nth(2).expect("no path given");

    let args = Cli {        // 手动解析参数
        pattern: pattern,
        path: std::path::PathBuf::from(path),
    };
}
