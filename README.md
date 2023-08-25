# 1 grrs 命令行工具

- CLI工具典型调用方式如下：

```
grrs foobar test.txt
```

- 程序名称后面的文本`foobar`、`test.txt`通常被称为 “命令行参数”（用字符串隔开），操作系统通常将其变换为字符串列表。

> 思考：
> 1. 如何解析命令，使其简单易用？
> 2. 告诉用户需要给出哪些参数以及文件类型？


---

# 2 使用 Rust 构建

- 从`cargo`开始：

```
cargo new grrs      # 新建项目
cargo run           # 运行项目
```

- `Cargo.toml`：包含我们项目的元数据的文件， 包括我们使用的依赖项/外部库的列表。
- `src/main.rs`：一个文件，它是我们的（主）二进制文件的入口点。


## 2.1 获取参数

- 标准库函数`std::env::args()`提供了参数迭代器：

```rust
use std::env::args;

let pattern = args().nth(1).expect("no pattern given");
let path = args().nth(2).expect("no path given");
```

## 2.2 给参数赋予数据类型

- CLI 的参数通常可以自定义其数据类型，例如 `grrs foobar test.txt` 中，`foobar` 是要查找的字符串，`test.txt` 是要查看的文件，在`src/main.rs`中编写：

```rust
struct Cli {
    pattern: String,            // 要查找的字符串
    path: std::path::PathBuf,   // 要查看的文件
}
```

- 以上定义了一个新的结构体，其中有两个用于存储数据的字段：`pattern`、`path`。

> 注意：`PathBuf`类似于`String`，但适用于跨平台工作的文件路径。

- 但是，我们仍然需要将程序时机参数进行转换，最基本的方式就是用如下代码块手动解析：

```rust
let args = Cli {        // 手动解析参数
    pattern: pattern,
    path: std::path::PathBuf::from(path),
};
```

> 手动解析很有效但是不方便，思考：
> 1. 如何处理参数`--pattern="foo"`、`--pattern "foo"`？
> 2. 如何处理参数`--help`？


## 2.3 使用 Clap 解析参数

- 调用`Clap`库是一个不错的方式。它是用于解析 CLI 参数最流行的库。其包括对子命令、shell完成和重要帮助消息的支持。

- 首先，在`Cargo.toml`文件里加入如下代码块：

```
[dependencies]
clap = {  version = "4.0", features = ["derive"] }
```

> 注意：
> 这是依赖自身库的 feature :
> 以上配置为 clap 依赖开启了 derive feature


- 还可以通过 default-features = false 来禁用依赖库，例如:

```
[dependencies]
flate2 = { version = "1.0.3", default-features = false, features = ["zlib"] }
```


---

# 3 Crates 国内镜像源加速

## 3.1 手动设置镜像
- 拉取 crates.io 仓库代码尤其慢，很多次超时导致引用库没法编译。
- [USTC 源官方教程](https://mirrors.ustc.edu.cn/help/crates.io-index.html)

1. 进入到当前用户目录下的路径 `~/.cargo/`；
2. 在 `.cargo` 文件夹下创建 `config` 文件；
3. 打开 `config` 文件输入内容：

```
[source.crates-io]
registry = "https://github.com/rust-lang/crates.io-index"
replace-with = 'tuna'

# ustc源有点问题
# [source.ustc]
# registry = "git://mirrors.ustc.edu.cn.crates.io-index"

# 用清华源
[source.tuna]
registry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git"

[net]
git-fetch-with-cli = true
```

## 3.2 使用 RsProxy 镜像
- [RsProxy 官方教程](http://rsproxy.cn/)

1. 设置 Rustup 镜像， 修改配置 `~/.zshrc` or `~/.bashrc` ；

```
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"

```

2. 重启终端 `source ~/.bashrc` ；

3. 设置 crates.io 镜像， 修改配置 `~/.cargo/config`，已支持git协议和sparse协议，>=1.68 版本建议使用 `sparse-index`，速度更快。

```
[source.crates-io]
replace-with = 'rsproxy-sparse'
[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"
[net]
git-fetch-with-cli = true
```
## 3.3 vscode 中修改 rust-crates 拓展
- 将 `https://api.crates-vsc.space` 改成 `https://index.crates.io`

---

# 4 Git

- 项目上传：

```
git init
git add ./
git branch -M main
git commit -m "first commit"
git push --set-upstream origin main
git remote add origin https://github.com/lancerstadium/grrs.git
git push -u origin main
```

---

# 5 参考文章

- [Rust crates 国内镜像源加速配置](https://zhuanlan.zhihu.com/p/126204128)
- [Vscode扩展crates挂了](https://rustcc.cn/article?id=c177b1fc-cc68-43de-9587-ddd199ed8169)
- [Rust 条件编译 feature](https://course.rs/cargo/reference/features/intro.html)