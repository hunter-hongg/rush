use rustyline::{Config, Editor, error::ReadlineError, history::FileHistory};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let blue = "\x1b[34m";
    let red = "\x1b[31m";
    let yellow = "\x1b[33m";
    let cyan = "\x1b[36m";
    let none = "\x1b[0;0m";
    let version = "0.0.1";
    let rush_type = "beta";
    // 创建自定义配置
    let config = Config::builder()
        .tab_stop(8)  // Tab宽度
        .indent_size(4)  // 缩进大小
        .build();
    
    // 创建带配置的编辑器
    let mut rl: Editor<(), FileHistory> = Editor::with_config(config)?;
    
    loop {
        println!("{}Rush Version {}{} Build Type {}{}", blue, version, yellow, rush_type, none);
        match rl.readline(&format!("$ ")) {
            Ok(line) => {
                let trimmed = line.trim();
                
                // 处理退出命令
                if trimmed == "exit" {
                    break;
                }
                
                // 处理空输入
                if trimmed.is_empty() {
                    continue;
                }
                
                println!("{}你输入了: {}{}", cyan, trimmed, none);
                
                // 添加到历史记录（可选）
                rl.add_history_entry(trimmed)?;
            },
            Err(ReadlineError::Interrupted) => {
                println!("{}中断 (Ctrl+C) - 输入 'exit' 退出{}", red, none);
            },
            Err(ReadlineError::Eof) => {
                println!("{}文件结束符 (Ctrl+D) - 退出程序{}", blue, none);
                break;
            },
            Err(err) => {
                println!("{}错误: {}{}", red, err, none);
                break;
            }
        }
    }
    
    Ok(())
}
