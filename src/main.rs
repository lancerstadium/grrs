#![allow(unused)]
use clap::{Parser, parser::Indices};
use anyhow::{Context, Result};

// 在文件中搜索字符串并显示包含该字符串的行
#[derive(Parser)]
struct Cli {
    pattern: String,            // 要查找的字符串
    path: std::path::PathBuf,   // 要查看的文件
}

fn main() -> Result<()>{
    // 自动解析参数到 Cli
    let args = Cli::parse(); 
    // 读取文件
    let content = std::fs::read_to_string(&args.path).with_context(|| format!("could not read fine `{}`!", args.path.to_string_lossy()))?;
    // 打印文件中含有目标值的每一行
    for line in content.lines() {
        if line.contains(&args.pattern) {
            println!("{}", line);
        }
    }
    Ok(())

}
