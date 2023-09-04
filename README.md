# 1 grrs 命令行工具

- 在本项目中，我们将设计一个 CLI 工具`grrs`，其主要可以进行文件内部文本的搜索（类似于`grep`）。

- CLI 工具典型调用方式如下：

```
grrs foobar test.txt
```

- 程序名称后面的文本`foobar`、`test.txt`通常被称为 “命令行参数”（用字符串隔开），操作系统通常将其变换为字符串列表。

> 思考：
> 1. 如何解析命令，使其简单易用？
> 2. 告诉用户需要给出哪些参数以及文件类型？


---

# 2 使用 Rust 构建

- rust安装：

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

- 从`cargo`开始：

```
cargo new grrs      # 新建项目
cargo run           # 运行项目
```

- `Cargo.toml`：包含我们项目的元数据的文件， 包括我们使用的依赖项/外部库的列表。
- `src/main.rs`：一个文件，它是我们的（主）二进制文件的入口点。
- `target/debug/`：生成可执行程序会在该目录下。


## 2.1 获取参数

- 标准库函数`std::env::args()`提供了参数迭代器：

```rust
use std::env::args;

let pattern = args().nth(1).expect("no pattern given");
let path = args().nth(2).expect("no path given");
```

- 或者这样引入`env`这位大哥，毕竟人家要出场多次的：

```rust
use std::env;
let args: Vec<String> = env::args().collect();
dbg!(args);
```

> 注意⚠️：
> 所有的用户输入都不可信！不可信！不可信！
> 原因是当传入的命令行参数包含非 `Unicode` 字符时
> `std::env::args` 会直接崩溃
> 建议大家使用`std::env::args_os`
> 该方法产生的数组将包含 `OsString` 类型
> 而不是之前的 `String` 类型
> 但是咱们不用

- `collect`并不是`env`包提供的，而是迭代器自带的方法。

- 存储参数：

```rust
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let query = &args[1];
    let file_path = &args[2];

    println!("Searching for {}", query);
    println!("In file {}", file_path);
}
```

## 2.2 文件读取

- 首先，通过 `use std::fs` 引入文件操作包，然后通过 `fs::read_to_string` 读取指定的文件内容：

```rust
use std::env;
use std::fs;

fn main() {
    // --省略之前的内容--
    println!("In file {}", file_path);

    let contents = fs::read_to_string(file_path)
        .expect("Should have been able to read the file");

    println!("With text:\n{contents}");
}
```

## 2.3 准备测试文件

- 准备`poem.txt`放入项目主目录下，与`src/`并列：

```txt
I'm nobody! Who are you?
Are you nobody, too?
Then there's a pair of us - don't tell!
They'd banish us, you know.

How dreary to be somebody!
How public, like a frog
To tell your name the livelong day
To an admiring bog!
```

- 测试命令：

```shell
cargo run -- the poem.txt
```

---

# 3 改进工程

## 3.1 增加模块化和错误处理

- 但凡稍微没那么糟糕的程序，都应该具有代码模块化和错误处理，不然连玩具都谈不上。梳理代码后，可以整理出如下四个改进点：
  1. **单一庞大的函数**。对于`grrs`而言，`main`函数执行两个任务：解析命令行参数和读取文件。但随着代码增加，其承载的功能也将快速增加。从工程角度来看，一个函数尽量才分出更小的功能单元，便于阅读和维护。
  2. **配置变量散乱**。当前`main`函数中的变量独立存在，可能被整个程序访问。我们可以将其整合进结构体中。
  3. **细化错误提示**。文件不存在、无权限等等都是可能的错误，一条大一统的消息无法给予用户更多的提示。
  4. 使用错误而非异常。如用户不给任何命令行参数，那我们的程序显然会无情崩溃，原因很简单：`index out of bounds`，一个数组访问越界的 `panic`。但问题来了，用户能看懂吗？因此需要增加合适的错误处理代码，来给予使用者给详细友善的提示。还有就是需要在一个统一的位置来处理所有错误，利人利己！


### 3.1.1 分离`main`函数

- Rust 社区给出了统一的处理`main`函数指导方案，这个方案叫做关注点分离（Separation of Concerns）：
  - 将程序分割为`main.rs`和`lib.rs`，并将程序的逻辑代码移动到后者内；
  - 从测试的角度而言，这种分离也非常合理： `lib.rs` 中的主体逻辑代码可以得到简单且充分的测试，至于 `main.rs` ？确实没办法针对其编写额外的测试代码，但是它的代码也很少啊，很容易就能保证它的正确性。
  - 命令行解析属于基本功能，不能属于逻辑代码的一部分。

- 梳理后`main`函数中应该包含的功能为：
  - 解析命令行参数
  - 初始化其他配置
  - 调用`lib.rs`中的`run`函数，来启动逻辑代码的运行
  - 如果`run`返回一个错误，则需要对该错误进行处理


- 接下来分离命令行解析。根据之前的分析，我们需要将命令行解析的代码分离到一个单独的函数，然后将该函数放置在`main.rs`中：

```rust
fn main() {
    let args: Vec<String> = env::args().collect();
    let (query, file_path) = parse_config(&args);

    // --省略--
}

fn parse_config(args: &[String]) -> (&str, &str) {
    let query = &args[1];
    let file_path = &args[2];

    (query, file_path)
}
```

- 经过分离后，之前的设计目标完美达成，即精简了 `main` 函数，又将配置相关的代码放在了 `main.rs` 文件里。
- 看起来貌似是杀鸡用了牛刀，但是重构就是这样，一步一步，踏踏实实的前行。

### 3.1.2 聚合配置变量

- 前文提到，配置变量并不适合分散的到处都是，因此使用一个结构体来统一存放是非常好的选择，这样修改后，后续的使用以及未来的代码维护都将更加简单明了。

```rust
fn main() {
    let args: Vec<String> = env::args().collect();

    let config = parse_config(&args);

    println!("Searching for {}", config.query);
    println!("In file {}", config.file_path);

    let contents = fs::read_to_string(config.file_path)
        .expect("Should have been able to read the file");

    // --snip--
}

struct Config {
    query: String,
    file_path: String,
}

fn parse_config(args: &[String]) -> Config {
    let query = args[1].clone();
    let file_path = args[2].clone();

    Config { query, file_path }
}

```

- 值得注意的是，`Config` 中存储的并不是 `&str` 这样的引用类型，而是一个 `String` 字符串，也就是 `Config` 并没有去借用外部的字符串，而是拥有内部字符串的所有权。`clone` 方法的使用也可以佐证这一点。

> clone 的得与失：
> 
> 在上面的代码中，除了使用 clone ，还有其它办法来达成同样的目的，但 clone 无疑是最简单的方法：直接完整的复制目标数据，无需被所有权、借用等问题所困扰，但是它也有其缺点，那就是有一定的性能损耗。
>
> 因此是否使用 clone 更多是一种性能上的权衡，对于上面的使用而言，由于是配置的初始化，因此整个程序只需要执行一次，性能损耗几乎是可以忽略不计的。
>
> 总之，判断是否使用 clone：
>
> - 是否严肃的项目，玩具项目直接用 clone 就行，简单不好吗？
> - 要看所在的代码路径是否是热点路径(hot path)，例如执行次数较多的显然就是热点路径，热点路径就值得去使用性能更好的实现方式。


- 继续优化，通过构造函数来初始化一个 `Config` 实例，而不是直接通过函数返回实例：

```rust
fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::new(&args);

    // --snip--
}

// --snip--

impl Config {
    fn new(args: &[String]) -> Config {
        let query = args[1].clone();
        let file_path = args[2].clone();

        Config { query, file_path }
    }
}
```

- 修改后，类似 `String::new` 的调用，我们可以通过 `Config::new` 来创建一个实例。


### 3.1.3 改进错误处理

#### a 主动 panic
- `panic` 的两种用法: 被动触发和主动调用。上面代码的方式很明显是被动触发，这种报错信息是不可控的，下面我们先改成主动调用的方式：

```rust
// in main.rs
 // --snip--
    fn new(args: &[String]) -> Config {
        if args.len() < 3 {
            panic!("not enough arguments");
        }
        // --snip--
```

- 不错，用户看到了更为明确的提示，但是还是有一大堆 debug 输出，这些我们其实是不想让用户看到的。这么看来，想要输出对用户友好的信息, `panic` 是不太适合的，它更适合告知开发者，哪里出现了问题。


#### b 返回 Result 替代 panic
- 那只能祭出之前学过的错误处理大法了，也就是返回一个 `Result`：成功时包含 `Config` 实例，失败时包含一条错误信息。
- 有一点需要额外注意下，从代码惯例的角度出发，`new` 往往不会失败，毕竟新建一个实例没道理失败，对不？因此修改为 `build` 会更加合适：

```rust
impl Config {
    fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        let query = args[1].clone();
        let file_path = args[2].clone();

        Ok(Config { query, file_path })
    }
}

```

- 这里的 `Result` 可能包含一个 `Config` 实例，也可能包含一条错误信息 `&static str`，不熟悉这种字符串类型的同学可以回头看看字符串章节，代码中的字符串字面量都是该类型，且拥有 `'static` 生命周期。

#### c 处理返回的 Result
- 接下来就是在调用 `build` 函数时，对返回的 `Result` 进行处理了，目的就是给出准确且友好的报错提示, 为了让大家更好的回顾我们修改过的内容，这里给出整体代码：
```rust
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 对 build 返回的 `Result` 进行处理
    let config = Config::build(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });


    println!("Searching for {}", config.query);
    println!("In file {}", config.file_path);

    let contents = fs::read_to_string(config.file_path)
        .expect("Should have been able to read the file");

    println!("With text:\n{contents}");
}

struct Config {
    query: String,
    file_path: String,
}

impl Config {
    fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        let query = args[1].clone();
        let file_path = args[2].clone();

        Ok(Config { query, file_path })
    }
}
```

- 上面代码有几点值得注意：
  - 当 `Result` 包含错误时，我们不再调用 `panic` 让程序崩溃，而是通过 `process::exit(1)` 来终结进程，其中 1 是一个信号值(事实上非 0 值都可以)，通知调用我们程序的进程，程序是因为错误而退出的。
  - `unwrap_or_else` 是定义在 `Result<T,E>` 上的常用方法，如果 `Result` 是 `Ok`，那该方法就类似 `unwrap`：返回 `Ok` 内部的值；如果是 `Err`，就调用闭包中的自定义代码对错误进行进一步处理。

> 综上可知，`config` 变量的值是一个 `Config` 实例，而 `unwrap_or_else` 闭包中的 `err` 参数，它的类型是 `'static str`，值是 `"not enough arguments"` 那个字符串字面量。

### 3.1.4 分离主体逻辑

- 接下来可以继续精简 `main` 函数，那就是将主体逻辑( 例如业务逻辑 )从 `main` 中分离出去，这样 `main` 函数就保留主流程调用，非常简洁。

```rust 
// in main.rs
fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    println!("Searching for {}", config.query);
    println!("In file {}", config.file_path);

    run(config);
}

fn run(config: Config) {
    let contents = fs::read_to_string(config.file_path)
        .expect("Should have been able to read the file");

    println!("With text:\n{contents}");
}

// --snip--
```

> 如上所示，`main` 函数仅保留主流程各个环节的调用，一眼看过去非常简洁清晰。


### 3.1.5 使用 ? 和特征对象返回错误

- 我们发现：`run` 函数没有错误处理，错误处理最好统一在一个地方完成，这样极其有利于后续的代码维护。

```rust
//in main.rs
use std::error::Error;

// --snip--

fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;

    println!("With text:\n{contents}");

    Ok(())
}
```

> 值得注意的是这里的 `Result<(), Box<dyn Error>>` 返回类型，首先我们的程序无需返回任何值，但是为了满足 `Result<T,E>` 的要求，因此使用了 `Ok(())` 返回一个单元类型 `()`。
>
> 最重要的是 `Box<dyn Error>`， 如果按照顺序学到这里，大家应该知道这是一个 `Error` 的特征对象：它表示函数返回一个类型，该类型实现了 `Error` 特征，这样我们就无需指定具体的错误类型，否则你还需要查看 `fs::read_to_string` 返回的错误类型。
>
> 简单来说，`fs::read_to_string`被强转为了`Box<dyn Error>`，用就是了。

- 先回忆下在 `build` 函数调用时，我们怎么处理错误的？然后与这里的方式做一下对比，没错 `if let` 的使用让代码变得更简洁，可读性也更加好，原因是，我们并不关注 `run` 返回的 `Ok` 值，因此只需要用 `if let` 去匹配是否存在错误即可：

```rust
fn main() {
    // --snip--

    println!("Searching for {}", config.query);
    println!("In file {}", config.file_path);

    if let Err(e) = run(config) {
        println!("Application error: {e}");
        process::exit(1);
    }
}
```

### 3.1.6 分离逻辑代码到库包

- 首先，创建一个`src/lib.rs`，将所有非`main`函数移动到其中：

```rust
use std::error::Error;
use std::fs;

pub struct Config {
    pub query: String,
    pub file_path: String,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        // --snip--
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    // --snip--
}

```

- 然后更改`src/main.rs`中代码：

```rust
use std::env;
use std::process;

use grrs::Config;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 对 build 返回的 `Result` 进行处理
    let config = Config::build(&args).unwrap_or_else(|err| {
        println!("Problem parsing arguments: {err}");
        process::exit(1);
    });


    println!("Searching for '{}'", config.query);
    println!("In file {}", config.file_path);

    if let Err(e) = grrs::run(config) {
        println!("Application error: {e}");
        process::exit(1);
    }
}

```

> 很明显，这里的 `grrs::run` 的调用，以及 `Config` 的引入，跟使用其它第三方包已经没有任何区别，也意味着我们成功的将逻辑代码放置到一个独立的库包中，其它包只要引入和调用就行。


## 3.2 测试驱动开发

> 在之前的章节中，我们完成了对项目结构的重构，并将进入逻辑代码编程的环节，但在此之前，我们需要先编写一些测试代码，也是最近颇为流行的测试驱动开发模式（TDD, Test Driven Development）：
> 1. 编写一个注定失败的测试，并且失败的原因和你指定的一样
> 2. 编写一个成功的测试
> 3. 编写你的逻辑代码，直到通过测试
> 
> 这三个步骤将在我们的开发过程中不断循环，直到所有的代码都开发完成并成功通过所有测试。

### 3.2.1 注定失败的测试用例
- 既然要添加测试，那之前的 `println!` 语句将没有大的用处，毕竟 `println!` 存在的目的就是为了让我们看到结果是否正确，而现在测试用例将取而代之。
- 接下来，在`lib.rs`文件中，添加`tests`模块和`test`函数：

```rust
// in lib.rs
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }
}

```

- 先添加一个简单的 `search` 函数实现，非常简单粗暴的返回一个空的数组，显而易见测试用例将成功通过，真是一个居心叵测的测试用例！
- 注意这里生命周期 'a 的使用，之前的章节有详细介绍，不太明白的同学可以回头看看。
- 它会失败！


### 3.2.2 务必成功的测试用例

- 接着就是测试驱动的第二步：编写注定成功的测试。当然，前提条件是实现我们的 `search` 函数。它包含以下步骤：
  1. 遍历迭代 contents 的每一行
  2. 检查该行内容是否包含我们的目标字符串
  3. 若包含，则放入返回值列表中，否则忽略
  4. 返回匹配到的返回值列表

#### a 遍历迭代每一行

- Rust 提供了一个很便利的 lines 方法将目标字符串进行按行分割：

```rust
// in lib.rs
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    for line in contents.lines() {
        // do something with line
    }
}
```

- 这里的 `lines` 返回一个迭代器，关于迭代器在后续章节会详细讲解，现在只要知道 `for` 可以遍历取出迭代器中的值即可。

#### b 在每一行中查询目标字符串

- 与之前的 `lines` 函数类似，Rust 的字符串还提供了 `contains` 方法，用于检查 `line` 是否包含待查询的 `query`：

```rust
// in lib.rs
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    for line in contents.lines() {
        if line.contains(query) {
            // do something with line
        }
    }
}
```

#### c 存储匹配到的结果

- 创建一个 `Vec` 动态数组，然后将查询到的每一个 `line` 推进数组中即可：

```rust
// in lib.rs
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();

    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }

    results
}
```

- 至此，`search` 函数已经完成了既定目标，为了检查功能是否正确，运行下我们之前编写的测试用例。

#### d 在 run 函数中调用 search 函数

- 如下：

```rust
// in src/lib.rs
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;

    for line in search(&config.query, &contents) {
        println!("{line}");
    }

    Ok(())
}
```

- 运行：

```shell
cargo run -- frog poem.txt              # 1
cargo run -- body poem.txt              # 3
cargo run -- monomorphization poem.txt  # 0
```


## 3.3 使用环境变量

- 在上一章节中，留下了一个悬念，该如何实现用户控制的大小写敏感，其实答案很简单，你在其它程序中肯定也遇到过不少，例如如何控制 panic 后的栈展开？ Rust 提供的解决方案是通过命令行参数来控制：

```rust
RUST_BACKTRACE=1 cargo run
```

- 与之类似，我们也可以使用环境变量来控制大小写敏感，例如：

```rust
IGNORE_CASE=1 cargo run -- to poem.txt
```

### 3.3.1 编写大小写不敏感的测试用例

- 还是遵循之前的规则：测试驱动，这次是对一个新的大小写不敏感函数进行测试 `search_case_insensitive`：

#### a 注定失败的用例

- 首先编写一个注定失败的用例：

```rust
// in src/lib.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_sensitive() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Duct tape.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }

    #[test]
    fn case_insensitive() {
        let query = "rUsT";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.
Trust me.";

        assert_eq!(
            vec!["Rust:", "Trust me."],
            search_case_insensitive(query, contents)
        );
    }
}

```

#### b 成功的用例

- 这里新增了一个 `case_insensitive` 测试用例，并对`search_case_insensitive` 进行了测试，实现其函数：

```rust
pub fn search_case_insensitive<'a>(
    query: &str,
    contents: &'a str,
) -> Vec<&'a str> {
    let query = query.to_lowercase();
    let mut results = Vec::new();

    for line in contents.lines() {
        if line.to_lowercase().contains(&query) {
            results.push(line);
        }
    }

    results
}
```

- 跟之前一样，但是引入了一个新的方法 `to_lowercase`，它会将 `line` 转换成全小写的字符串。`query` 现在是 `String` 类型，而不是之前的 `&str`，因为 `to_lowercase` 返回的是 `String`。


#### c 在 run 中调用

- 最后一步，在 `run` 中调用新的搜索函数。但是在此之前，要新增一个配置项，用于控制是否开启大小写敏感。

```rust
// in lib.rs
pub struct Config {
    pub query: String,
    pub file_path: String,
    pub ignore_case: bool,
}
```

- 修改`run`中开启敏感检查：

```rust
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(config.file_path)?;

    let results = if config.ignore_case {
        search_case_insensitive(&config.query, &contents)
    } else {
        search(&config.query, &contents)
    };

    for line in results {
        println!("{line}");
    }

    Ok(())
}

```

- 现在的问题来了，该如何控制这个配置项呢。这个就要借助于章节开头提到的环境变量，好在 Rust 的 `env` 包提供了相应的方法。

```rust
use std::env;
// --snip--

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        let query = args[1].clone();
        let file_path = args[2].clone();

        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            query,
            file_path,
            ignore_case,
        })
    }
}

```

- `env::var` 没啥好说的，倒是 `is_ok` 值得说道下。该方法是 `Result` 提供的，用于检查是否有值，有就返回 `true`，没有则返回 `false`，刚好完美符合我们的使用场景，因为我们并不关心 `Ok<T>` 中具体的值。

- 运行：

```shell
cargo run -- to poem.txt
IGNORE_CASE=1 cargo run -- to poem.txt
```

- 大小写不敏感后，查询到的内容明显多了很多，也很符合我们的预期。

> 小作业：
> 同时使用命令行参数和环境变量的方式来控制大小写不敏感，其中环境变量的优先级更高，也就是两个都设置的情况下，优先使用环境变量的设置。


## 3.4 重定向错误信息输出

- 迄今为止，所有的输出信息，无论 `debug` 还是 `error` 类型，都是通过 `println!` 宏输出到终端的标准输出(`stdout`)，但是对于程序来说，错误信息更适合输出到标准错误输出(`stderr`)。
- 这样修改后，用户就可以选择：
  - 将普通的日志类信息输出到日志文件 1；
  - 然后将错误信息输出到日志文件 2；
  - 甚至还可以输出到终端命令行。


### 3.4.1 目前的错误输出位置

- 我们先来观察下，目前的输出信息包括错误，是否是如上面所说，都写到标准错误输出。
- 测试方式很简单，将标准错误输出的内容重定向到文件中，看看是否包含故意生成的错误信息即可。

```shell
cargo run > output.txt
```

- 首先，这里的运行没有带任何参数，因此会报出类如文件不存在的错误，其次，通过 `>` 操作符，标准输出上的内容被重定向到文件 `output.txt` 中，不再打印到控制上，输出到项目主目录下。

### 3.4.2 标准错误输出 stderr
- 将错误信息重定向到 `stderr` 很简单，只需在打印错误的地方，将 `println!` 宏替换为 `eprintln!`即可。

```rust
fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    if let Err(e) = minigrep::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
```

- 再次运行：可以看到，日志信息成功的重定向到 `output.txt` 文件中，而错误信息由于 `eprintln!` 的使用，被写入到标准错误输出中，默认还是输出在控制台中。

- 试试无错误情况：

```rust
cargo run -- to poem.txt > output.txt
```

- 至此，简易搜索程序 `grrs` 已经基本完成，下一章节将使用迭代器进行部分改进。


## 3.5 使用迭代器改进程序

- 在之前的 `minigrep` 中，功能虽然已经 ok，但是一些细节上还值得打磨下，下面一起看看如何使用迭代器来改进 `Config::build` 和 `serach` 的实现。

### 3.5.1 移除 clone 的使用

- 虽然之前有讲过为什么这里可以使用 `clone`，但是也许总有同学心有芥蒂，毕竟程序员嘛，都希望代码处处完美，而不是丑陋的处处妥协。

- 之前的代码大致长这样，两行 clone 着实有点啰嗦，好在，在学习完迭代器后，我们知道了 `build` 函数实际上可以直接拿走迭代器的所有权，而不是去借用一个数组切片 `&[String]`。

```rust
impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        let query = args[1].clone();
        let file_path = args[2].clone();

        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            query,
            file_path,
            ignore_case,
        })
    }
}
```

### 3.5.2 直接使用返回的迭代器

- 在之前的实现中，我们的 `args` 是一个动态数组:

```rust
fn main() {
    let args: Vec<String> = env::args().collect();

    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });
    // --snip--
}
```

- 当时还提到了 `collect` 方法的使用，相信大家学完迭代器后，对这个方法会有更加深入的认识。
- 现在呢，无需数组了，直接传入迭代器即可：

```rust
fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });
    // --snip--
}
```

- 原因是 `env::args` 可以直接返回一个迭代器，再作为 `Config::build` 的参数传入。改写`lib.rs/build`方法：

```rust
impl Config {
    pub fn build(
        mut args: impl Iterator<Item = String>,
    ) -> Result<Config, &'static str> {
        // --snip--
```

- 为了可读性和更好的通用性，这里的 `args` 类型并没有使用本身的 `std::env::Args` ，而是使用了特征约束的方式来描述 `impl Iterator<Item = String>`，这样意味着 arg 可以是任何实现了 `String` 迭代器的类型。

> 注意，由于迭代器的所有权已经转移到 `build` 内，因此可以直接对其进行修改，这里加上了 `mut` 关键字。

### 3.5.3 移除数组索引的使用

- 数组索引会越界，为了安全性和简洁性，使用 `Iterator` 特征自带的 `next` 方法是一个更好的选择:

```rust
impl Config {
    pub fn build(
        mut args: impl Iterator<Item = String>,
    ) -> Result<Config, &'static str> {
        // 第一个参数是程序名，由于无需使用，因此这里直接空调用一次
        args.next();

        let query = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string"),
        };

        let file_path = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file path"),
        };

        let ignore_case = env::var("IGNORE_CASE").is_ok();

        Ok(Config {
            query,
            file_path,
            ignore_case,
        })
    }
}
```

- 上面使用了迭代器和模式匹配的代码，看上去是不是很 Rust？


### 3.5.4 使用迭代器适配器让代码更简洁

- 为了帮大家更好的回忆和对比，之前的 `search` 长这样：
```rust
// in lib.rs
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();

    for line in contents.lines() {
        if line.contains(query) {
            results.push(line);
        }
    }
    results
}
```

- 引入了迭代器后，就连古板的 `search` 函数也可以变得更 `rusty` 些:

```rust
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    contents
        .lines()
        .filter(|line| line.contains(query))
        .collect()
}
```

- Let's Rock，这种一行到底的写法有时真的让人沉迷。





## 3.6 使用 crates 重构项目

### 3.6.1 给参数赋予数据类型

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

### 3.6.2 使用 Clap 解析参数

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

- 现在我们可以在代码中编写`use clap::Parser;`，修改如下：

```rust
use clap::Parser;

// 在文件中搜索模式并显示包含该模式的行。
#[derive(Parser)]
struct Cli {
    pattern: String,            // 要查找的字符串
    path: std::path::PathBuf,   // 要查看的文件
}
```

> 注意：
> 将自定义属性添加到字段中，如：
> 想将此字段用于-o或--output之后的参数
> 可以添加 `#[arg(short = 'o', long = "output")]`

- 在`main()`中解析参数：

```rust
fn main() {
    let args = Cli::parse(); // 自动解析参数到 Cli
}
```

- 运行测试`cargo run`、`cargo run -- some-pattern some-file`


### 3.6.3 文件读入

- 从打开我们收到的文件开始：

```rust
    // 读取文件
    let content = std::fs::read_to_string(&args.path).expect("Could not read file!");
```

- 迭代文件每一行，循环打印：

```rust
    // 打印文件每一行
    for line in content.lines() {
        if line.contains(&args.pattern) {
            println!("{}", line);
        }
    }
```

- 现在代码就像；

```rust
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
```

- 尝试`cargo run -- main src/main.rs`，现在它可以成功查找文件中的第一个匹配字符串。

> 注意：
> 这不是最好的实现
> 它将整个文件读入内存
> 找到一种优化方法
> 一个想法是使用`aBufReader`替代`read_to_string`



### 3.6.4 错误处理

- 如果`read_to_string`返回错误类型`std::io::Error`，需要进行处理：

```rust
    let result = std::fs::read_to_string("test.txt");
    match result {  // 错误处理：没找到文件则报错
        Ok(content) => { println!("File content: {}", content); }
        Err(error) => { println!("Oh noes: {}", error); }
    }
```

- 取用`content`：

```rust
    let result = std::fs::read_to_string("test.txt");
    let content = match result {  
        // 错误处理：没找到文件则报错
        Ok(content) => { content },
        Err(error) => { panic!("Can't deal with {}, just exit here", error); }
    };
    println!("file content: {}", content);
```

- 不使用`panic!`，返回类型为`Result!`：

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = std::fs::read_to_string("test.txt");
    let content = match result {
        Ok(content) => { content },
        Err(error) => { return Err(error.into()); }
    };
    println!("file content: {}", content);
    Ok(())
}
```

- 使用`?`，Rust将在内部将此扩展为与我们刚刚编写的`match`非常相似的东西，十分简洁：

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("test.txt")?;
    println!("file content: {}", content);
    Ok(())
}
```

> 注意：
> `main`函数中的错误类型是`Box<dyn std::error::Error>`
> 但我们在上面看到，`read_to_string`返回`std::io::Error`
> 这是因为`?`拓展到转换错误类型的代码
> `Box<dyn std::error::Error>`是一个可以包含任何类型
> 实现标准`Error trait`的`Box`
> 意味着基本上所有错误都可以放入这个`Box`里。

- 提供错误内容，例如，可以创建自己的错误类型，然后使用它来构建自定义错误消息：

```rust
#[derive(Debug)]
struct CustomError(String);

fn main() -> Result<(), CustomError> {
    let path = "test.txt";
    let content = std::fs::read_to_string(path)
        .map_err(|err| CustomError(format!("Error reading `{}`: {}", path, err)))?;
    println!("file content: {}", content);
    Ok(())
}
```

- 其有一个问题：不存储原始错误，只存储其字符串表示。`anyhow`库有一个解决方案：与`CustomError`类似，其`Context`特征用于添加描述并保留原始错误信息。将`anyhow = "1.0.75"`加入`Cargo.toml`，在`src/main.rs`中使用：

```rust
#![allow(unused)]
use clap::Parser;
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

```

### 3.6.5 信息输出

- 打印错误应通过`stderr`完成：

```rust
eprintln!("This is an error! :(");
```

- `stdout`获得锁定并使用`writeln!`直接打印它。

```rust
use std::io::{self, Write};

let stdout = io::stdout(); // get the global stdout entity
let mut handle = stdout.lock(); // acquire a lock on it
writeln!(handle, "foo: {}", 42); // add `?` if you care about errors here
```

- 显示进度条：一些CLI应用程序运行不到一秒钟，另一些则需要几分钟或几个小时。尝试打印易于使用的状态更新。

```rust
fn main() {
    let pb = indicatif::ProgressBar::new(100);
    for i in 0..100 {
        do_hard_work();
        pb.println(format!("[+] finished #{}", i));
        pb.inc(1);
    }
    pb.finish_with_message("done");
}

```

- 记录（Record）：添加一些日志语句：错误、警告、信息、调试和跟踪（错误的优先级最高，跟踪最低）。
- 需要准备：
  - 日志箱（Log Box）：包含以日志级别命名的宏
  - 适配器（Adapters）：实际将日志输出写入有用位置，不仅可以使用它们将日志写入终端，还可以写入syslog或中央日志服务器。

- 使用 `isenv_logger` 的简单适配器：被称为“env”记录器，您可以使用环境变量来指定要记录应用程序的哪些部分（以及要在哪个级别记录它们）。由于库也可以使用log，因此也可以轻松配置其日志输出：

```rust
use log::{info, warn};

fn main() {
    env_logger::init();
    info!("starting up");
    warn!("oops, nothing implemented!");
}

```

- 假设此文件为`src/bin/output-log.rs`，在`Linux`和`macOS`上，您可以像这样运行它：
```shell
env RUST_LOG=info cargo run --bin output-log
```

- `RUST_LOG`是可用于设置日志设置的环境变量的名称，`env_logger`还包含一个构建器，因此可以以编程方式调整这些设置。

---

# 4 Crates 国内镜像源加速

## 4.1 手动设置镜像
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

## 4.2 使用 RsProxy 镜像
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
## 4.3 vscode 中修改 rust-crates 拓展
- 将 `https://api.crates-vsc.space` 改成 `https://index.crates.io`



---

# 5 Git

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

# 6 参考文章

- [Rust构建命令行](https://rust-cli.github.io/book/tutorial/cli-args.html)
- [Rust crates国内镜像源加速配置](https://zhuanlan.zhihu.com/p/126204128)
- [Vscode扩展crates挂了](https://rustcc.cn/article?id=c177b1fc-cc68-43de-9587-ddd199ed8169)
- [Rust语言圣经-条件编译feature](https://course.rs/cargo/reference/features/intro.html)
- [Pigcha-Linux CLI使用](http://202.81.231.183:8081/misc/linux_tutorial)