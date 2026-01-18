use rustyline::{Config, Editor, error::ReadlineError, history::FileHistory};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建自定义配置
    let config = Config::builder()
        .tab_stop(8)  // Tab宽度
        .indent_size(4)  // 缩进大小
        .build();
    
    // 创建带配置的编辑器
    let mut rl: Editor<(), FileHistory> = Editor::with_config(config)?;
    
    loop {
        match rl.readline("$ ") {
            Ok(line) => {
                let trimmed = line.trim();
                
                // 处理退出命令
                if trimmed == "exit" || trimmed == "quit" {
                    println!("退出程序");
                    break;
                }
                
                // 处理空输入
                if trimmed.is_empty() {
                    continue;
                }
                
                println!("你输入了: {}", trimmed);
                
                // 添加到历史记录（可选）
                rl.add_history_entry(trimmed)?;
            },
            Err(ReadlineError::Interrupted) => {
                println!("中断 (Ctrl+C) - 输入 'exit' 或 'quit' 退出");
            },
            Err(ReadlineError::Eof) => {
                println!("文件结束符 (Ctrl+D) - 退出程序");
                break;
            },
            Err(err) => {
                println!("错误: {}", err);
                break;
            }
        }
    }
    
    Ok(())
}
