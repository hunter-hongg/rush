use std::process::exit;

use rustyline::{Config, Editor, error::ReadlineError, history::FileHistory};


const BLUE: &str = "\x1b[34m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const PURPLE: &str = "\x1b[35m";
const COLOR_NULL: &str = "\x1b[0;0m";
const VERSION: &str= "0.0.1";
const RUSH_TYPE: &str = "beta";

fn exec_exit(line: bool) {
    if line {
        println!();
    }
    println!("{}退出Rush.{}", CYAN, COLOR_NULL);
    exit(0);
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
        PURPLE,
        VERSION, RUSH_TYPE, 
        COLOR_NULL,
    );
}

fn exec_help() {
    print!(
        "\
        {}\n\
        Rush命令帮助\n\
        Rush 版本 {} 构建类型 {} \n\
        内置命令: \n\
        help -------------------------- 输出此帮助\n\
        exit -------------------------- 退出Rush\n\
        welcome ----------------------- 输出欢迎页面\n\
        命令别名: \n\
        welc -------------------------- welcome的别名\n\
        {}\
        ",
        CYAN, 
        VERSION, RUSH_TYPE,
        COLOR_NULL,
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    exec_welcome();
    
    // 创建自定义配置
    let config = Config::builder()
        .tab_stop(8)  // Tab宽度
        .indent_size(4)  // 缩进大小
        .build();
    
    // 创建带配置的编辑器
    let mut rl: Editor<(), FileHistory> = Editor::with_config(config)?;

    loop {
        println!();
        println!("{}Rush VERSION {}{} BUILD TYPE {}{}", BLUE, VERSION, YELLOW, RUSH_TYPE, COLOR_NULL);
        match rl.readline(&format!("{}$ {}", PURPLE, COLOR_NULL)) {
            Ok(line) => {
                let trimmed = line.trim();
                
                match trimmed {
                    "" => continue, 
                    "exit" => exec_exit(true),
                    "help" => exec_help(),
                    "welc" | "welcome" => exec_welcome(),
                    _ => println!("{}未知命令: {}{}", RED, trimmed, COLOR_NULL),
                }
                rl.add_history_entry(trimmed)?;
            },
            Err(ReadlineError::Interrupted) => {
                println!("{}中断 (Ctrl+C) - 输入 'exit' 退出{}", RED, COLOR_NULL);
            },
            Err(ReadlineError::Eof) => {
                println!();
                println!("{}发现文件结束符EOF.{}", CYAN, COLOR_NULL);
                exec_exit(false);
                break;
            },
            Err(err) => {
                println!("{}错误: {}{}", RED, err, COLOR_NULL);
                break;
            }
        }
    }
    
    Ok(())
}
