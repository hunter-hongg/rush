use std::process::exit;

use rustyline::{Config, Editor, error::ReadlineError, history::FileHistory};

const BLUE: &str = "\x1b[34m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const PURPLE: &str = "\x1b[35m";
const COLOR_NULL: &str = "\x1b[0;0m";
const VERSION: &str = "0.0.1";
const RUSH_TYPE: &str = "beta";

fn get_home() -> String {
    match std::env::home_dir() {
        Some(x) => format!("{}", x.display()),
        None => "".to_string(),
    }
}

fn get_cwd() -> String {
    match std::env::current_dir() {
        Ok(_) => {
            format!("{}工作目录 {}{}", CYAN, get_cwd_raw(), COLOR_NULL)
        },
        Err(_) => format!("{}错误的工作目录{}", RED, COLOR_NULL),
    }
}

fn get_cwd_raw() -> String {
    match std::env::current_dir() {
        Ok(p) => {
            let pwd = format!("{}", p.display());
            pwd.replace(&get_home(), "~")
        },
        Err(_) => "".to_string(),
    }
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
            },
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

fn exec_cd(args: Vec<&str>) {
    let mut flags: Vec<&str> = vec![];
    let mut path: &str = "";
    for i in args {
        if i.starts_with("-") {
            flags.push(i);
        } else {
            if path == "" {
                path = i;
            }
        }
    }
    match std::env::set_current_dir(path) {
        Err(e) => println!("{}cd失败: {}{}", RED, e, COLOR_NULL),
        _ => return,
    }
}

fn exec_pwd(args: Vec<&str>) {
    let mut flags: Vec<&str> = vec![];
    for i in args {
        if i.starts_with("-") {
            flags.push(i);
        } 
    }
    println!("{}", get_cwd_raw());
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
    let args: Vec<&str>;
    if trimmed_to_list.len() > 1 {
        args = trimmed_to_list[1..].to_vec();
    } else {
        args = vec![];
    }
    match command {
        "" => return,
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
            "{}Rush 版本 {}{} 构建版本 {} {} {}",
            BLUE, VERSION, YELLOW, RUSH_TYPE, get_cwd(), COLOR_NULL
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
