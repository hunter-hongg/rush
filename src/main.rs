use std::cell::RefCell;
use std::process::exit;

use rustyline::{Config, Editor, error::ReadlineError, history::FileHistory};
use users::{get_user_by_name, os::unix::UserExt};

const BLUE: &str = "\x1b[34m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const PURPLE: &str = "\x1b[35m";
const COLOR_NULL: &str = "\x1b[0;0m";
const VERSION: &str = "0.0.1";
const RUSH_TYPE: &str = "beta";

// 使用线程本地存储来保存逻辑路径栈
thread_local! {
    static LOGICAL_PATH_STACK: RefCell<Option<Vec<String>>> = const {RefCell::new(None)};
}

fn get_home() -> String {
    match std::env::home_dir() {
        Some(x) => format!("{}", x.display()),
        None => "".to_string(),
    }
}

fn get_cwd() -> String {
    let res = logical_path_stack_beautiful().replace(&get_home(), "~");
    format!("{}{}{}", CYAN, res, COLOR_NULL)
}

fn exec_exit(line: bool, args: Vec<&str>) {
    if line {
        println!();
    }
    println!("{}退出Rush.{}", CYAN, COLOR_NULL);
    if args.is_empty() {
        exit(0);
    } else {
        match args[0].parse::<i32>() {
            Ok(n) => {
                exit(n);
            }
            Err(_) => exit(0),
        }
    }
}

fn exec_welcome() {
    print!(
        "\
        {}\n\
        欢迎来到Rush, 一个类似fish但POSIX兼容的Shell\n\
        Rush 版本 {} 构建类型 {} \n\
        输入 'help' 获取帮助, 输入 'welc' 显示此内容. \n\
        {}\
        ",
        PURPLE, VERSION, RUSH_TYPE, COLOR_NULL,
    );
}

fn get_specific_user_home(username: &str) -> Option<String> {
    get_user_by_name(username).and_then(|user| user.home_dir().to_str().map(String::from))
}

fn replace_before_first(input: &str, delimiter: char, replacement: &str) -> String {
    if let Some(pos) = input.find(delimiter) {
        format!("{}{}", replacement, &input[pos..])
    } else {
        input.to_string()
    }
}

fn substring_between(start_pos: usize, end_char: char, s: &str) -> Option<&str> {
    // 检查起始位置是否有效
    if start_pos >= s.len() || !s.is_char_boundary(start_pos) {
        return None;
    }

    // 从指定位置开始查找结束字符
    let substring = &s[start_pos..];
    let relative_pos = substring.find(end_char)?;

    // 返回不包含结束字符的子串
    Some(&substring[..relative_pos])
}

// 全局变量用于存储逻辑路径栈（符号链接路径）
fn exec_cd(args: Vec<&str>) {
    let mut flags: Vec<&str> = vec![];
    let mut path: &str = "";
    let pathstring: String;
    for i in args {
        if i.starts_with("-") {
            flags.push(i);
        } else if path.is_empty() {
            path = i;
        }
    }
    if path.is_empty() {
        path = "~";
    }
    if path.starts_with("~") {
        if path.len() == 1 {
            pathstring = std::env::home_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            path = &pathstring;
        } else if path.chars().nth(1) == Some('/') {
            pathstring = std::env::home_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
                + &path[1..];
            path = &pathstring;
        } else if path.contains("/") {
            pathstring = if let Some(username) = substring_between(1, '/', path) {
                if let Some(user_home) = get_specific_user_home(username) {
                    replace_before_first(path, '/', &user_home)
                } else {
                    path.to_string() // 保持原样
                }
            } else {
                path.to_string() // 保持原样
            };
            path = &pathstring;
        } else {
            let username = &path[1..];
            if let Some(user_home) = get_specific_user_home(username) {
                pathstring = user_home;
            } else {
                pathstring = "".to_string();
            }
            path = &pathstring;
        }
    }

    // 检查是否使用了-P标志（解析符号链接）
    let use_physical_path = flags.iter().any(|&f| f.contains('P'));

    // 从线程本地存储获取旧的逻辑路径栈
    let old_logical_path_stack = LOGICAL_PATH_STACK.with(|stack| stack.borrow().clone());

    // 根据-P标志决定是否解析符号链接
    let actual_path = if use_physical_path {
        match std::fs::canonicalize(path) {
            Ok(canonical_path) => canonical_path,
            Err(e) => {
                println!("{}cd {}失败: {}{}", RED, path, e, COLOR_NULL);
                return;
            }
        }
    } else {
        std::path::PathBuf::from(path)
    };

    if let Err(e) = std::env::set_current_dir(&actual_path) {
        println!("{}cd {}失败: {}{}", RED, path, e, COLOR_NULL);
        return;
    }

    if use_physical_path {
        LOGICAL_PATH_STACK.with(|stack| {
            *stack.borrow_mut() = Some(vec![
                std::env::current_dir()
                    .unwrap_or(std::path::PathBuf::from("/"))
                    .display()
                    .to_string()
                    .split('/')
                    .collect(),
            ]);
        });
    } else {
        // 按照算法解析path并更新LOGICAL_PATH_STACK
        let new_path_stack = process_logical_path_stack(old_logical_path_stack, path);
        LOGICAL_PATH_STACK.with(|stack| *stack.borrow_mut() = Some(new_path_stack));
    }
}

// 按照算法解析路径，返回路径栈
fn process_logical_path_stack(
    current_path_stack_raw: Option<Vec<String>>,
    new_path: &str,
) -> Vec<String> {
    let path_parts: Vec<&str> = new_path.split('/').collect();
    let mut temp_stack: Vec<String> = Vec::new();
    let is_absolute = new_path.starts_with('/');
    let mut current_path_stack = current_path_stack_raw.clone().unwrap_or_default();

    // 遍历分割后的列表
    for part in path_parts {
        match part {
            "." | "" => continue, // 忽略.和空字符串
            ".." => {
                // ..，临时栈出栈元素，没有元素保持不变
                if !temp_stack.is_empty() {
                    temp_stack.pop();
                } else {
                    // 如果临时栈为空，从当前路径栈中弹出一个元素（如果存在）
                    if !is_absolute && !current_path_stack.is_empty() {
                        current_path_stack.pop();
                    }
                }
            }
            _ => {
                // 都不是，临时栈入栈一个元素
                temp_stack.push(part.to_string());
            }
        }
    }

    // 判断是否为绝对路径
    if is_absolute {
        // 以/开头，直接使用临时栈作为新的路径栈
        temp_stack
    } else {
        // 不以/开头，基于当前路径栈构建新路径栈
        let mut result_stack = current_path_stack;

        // 将临时栈中的元素追加到结果栈
        result_stack.extend(temp_stack);
        result_stack
    }
}

fn logical_path_stack_beautiful() -> String {
    LOGICAL_PATH_STACK.with(|stack| match stack.borrow().as_ref() {
        Some(path_stack) => {
            let mut res = String::new();
            for path in path_stack.iter() {
                if !res.ends_with('/') && path != "/" {
                    res += "/";
                }
                res += path;
            }
            res
        }
        None => std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
    })
}

fn exec_pwd(args: Vec<&str>) {
    let mut use_physical_path = false; // 对应 -P 参数

    for arg in args {
        if arg.starts_with("-") && arg.contains('P') {
            use_physical_path = true;
        }
    }

    // 如果同时指定了 -L 和 -P，则 -P 优先（这是标准行为）
    if use_physical_path {
        // 使用物理路径（解析符号链接）
        match std::env::current_dir() {
            Ok(path_buf) => {
                // canonicalize会解析符号链接并返回物理路径
                match path_buf.canonicalize() {
                    Ok(canonical_path) => println!("{}", canonical_path.display()),
                    Err(_) => {
                        // 如果canonicalize失败，回退到普通路径
                        println!("{}", path_buf.display());
                    }
                }
            }
            Err(e) => println!("{}pwd 失败: {}{}", RED, e, COLOR_NULL),
        }
    } else {
        // 对于 -L 或默认情况，显示逻辑路径（包含符号链接）
        LOGICAL_PATH_STACK.with(|stack| {
            // 根据POSIX标准，-L是默认行为，所以我们优先考虑逻辑路径
            // 如果有记录的逻辑路径栈，就打印它；否则使用当前目录
            match stack.borrow().as_ref() {
                Some(path_stack) => {
                    let logical_path = if path_stack.is_empty() {
                        "/".to_string() // 根据POSIX标准，根目录是"/"
                    } else {
                        logical_path_stack_beautiful()
                    };
                    println!("{}", logical_path);
                }
                None => {
                    // 如果没有记录逻辑路径，则使用当前目录（这应该是物理路径）
                    match std::env::current_dir() {
                        Ok(path_buf) => println!("{}", path_buf.display()),
                        Err(e) => println!("{}pwd 失败: {}{}", RED, e, COLOR_NULL),
                    }
                }
            }
        });
    }
}

fn exec_help() {
    print!(
        "\
        {}\n\
        Rush命令帮助\n\
        Rush 版本 {} 构建类型 {} \n\
        ===基础命令========================================\n\
        help -------------------------- 输出此帮助\n\
        exit -------------------------- 退出Rush\n\
        exit -------------------------- 带错误码退出: exit <code>\n\
        welcome ----------------------- 输出欢迎页面\n\
        ===目录浏览========================================\n\
        cd ---------------------------- 更改工作目录\n\
        pwd --------------------------- 输出当前工作目录\n\
        ===命令别名========================================\n\
        welc -------------------------- welcome的别名\n\
        chdir ------------------------- cd的别名\n\
        curdir ------------------------ pwd的别名\n\
        {}\
        ",
        CYAN, VERSION, RUSH_TYPE, COLOR_NULL,
    )
}

fn parse(trimmed: &str) {
    let trimmed_to_list: Vec<&str> = trimmed.split(' ').collect();
    if trimmed_to_list.is_empty() {
        return;
    }
    let command = trimmed_to_list[0];
    let args: Vec<&str> = if trimmed_to_list.len() > 1 {
        trimmed_to_list[1..].to_vec()
    } else {
        vec![]
    };
    match command {
        "" => (),
        "exit" => exec_exit(true, args),
        "help" => exec_help(),
        "welc" | "welcome" => exec_welcome(),
        "cd" | "chdir" => exec_cd(args.clone()),
        "pwd" | "curdir" => exec_pwd(args.clone()),
        _ => println!("{}未知命令: {}{}", RED, trimmed, COLOR_NULL),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    exec_welcome();

    LOGICAL_PATH_STACK.with(|stack| {
        *stack.borrow_mut() = Some(
            std::env::current_dir()
                .unwrap()
                .iter()
                .map(|os_str| os_str.to_string_lossy().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        );
    });

    // 创建自定义配置
    let config = Config::builder()
        .tab_stop(8) // Tab宽度
        .indent_size(4) // 缩进大小
        .build();

    // 创建带配置的编辑器
    let mut rl: Editor<(), FileHistory> = Editor::with_config(config)?;

    loop {
        println!();
        println!(
            "{}Rush {}{} {} {} {}",
            BLUE,
            VERSION,
            YELLOW,
            RUSH_TYPE,
            get_cwd(),
            COLOR_NULL
        );
        match rl.readline(&format!("{}$ {}", PURPLE, COLOR_NULL)) {
            Ok(line) => {
                let trimmed = line.trim();
                parse(trimmed);
                rl.add_history_entry(trimmed)?;
            }
            Err(ReadlineError::Interrupted) => {
                println!("{}中断 (Ctrl+C) - 输入 'exit' 退出{}", RED, COLOR_NULL);
            }
            Err(ReadlineError::Eof) => {
                println!();
                println!("{}发现文件结束符EOF.{}", CYAN, COLOR_NULL);
                exec_exit(false, vec![]);
                break;
            }
            Err(err) => {
                println!("{}错误: {}{}", RED, err, COLOR_NULL);
                break;
            }
        }
    }

    Ok(())
}
