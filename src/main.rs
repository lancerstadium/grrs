#![allow(unused)]
use clap::Parser;

// 在文件中搜索字符串并显示包含该字符串的行
#[derive(Parser)]
struct Cli {
    pattern: String,            // 要查找的字符串
    path: std::path::PathBuf,   // 要查看的文件
}

fn main() {
    // 自动解析参数到 Cli
    let args = Cli::parse(); 
    // 读取文件
    let content = std::fs::read_to_string(&args.path).expect("Could not read file!");
    // 打印文件每一行
    for line in content.lines() {
        if line.contains(&args.pattern) {
            println!("{}", line);
        }
    }
}
