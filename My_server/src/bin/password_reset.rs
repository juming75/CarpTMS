//! / 密码重置工具
// 用于为用户生成Argon2哈希密码

use std::env;
use std::io::{self, Write};

// 引入密码哈希模块
// 注意:这个文件应该放在 My_server/src/utils/ 或者作为一个独立的工具

#[path = "../utils/password.rs"]
mod password;

fn main() {
    println!("===============================================");
    println!("   CarpTMS 密码重置工具");
    println!("===============================================\n");

    // 检查命令行参数
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "hash" => {
            if args.len() < 3 {
                println!("错误: 请提供要哈希的密码");
                println!("用法: cargo run --bin password_reset -- hash <password>");
                return;
            }
            hash_password(&args[2]);
        }
        "batch" => {
            batch_generate_hashes();
        }
        "check" => {
            // 检查数据库中的密码格式(需要数据库连接)
            println!("提示: 使用 reset_passwords.sql 脚本检查数据库密码格式");
        }
        _ => {
            println!("错误: 未知命令 '{}'", args[1]);
            print_usage();
        }
    }
}

fn print_usage() {
    println!("用法:");
    println!("  cargo run --bin password_reset -- hash <password>");
    println!("      生成指定密码的Argon2哈希");
    println!();
    println!("  cargo run --bin password_reset -- batch");
    println!("      批量生成常用密码的哈希");
    println!();
    println!("  cargo run --bin password_reset -- check");
    println!("      检查密码格式提示");
    println!();
    println!("示例:");
    println!("  cargo run --bin password_reset -- hash admin123");
    println!("  cargo run --bin password_reset -- batch");
}

fn hash_password(password: &str) {
    println!("正在为密码生成哈希...");
    println!("密码: {}", password);
    println!();

    match password::hash_password(password) {
        Ok(hash) => {
            println!("✓ 哈希生成成功！");
            println!();
            println!("Argon2 哈希值:");
            println!("{}", hash);
            println!();
            println!("SQL 更新语句:");
            println!("UPDATE users SET password = '{}', update_time = CURRENT_TIMESTAMP WHERE user_name = 'username';", hash);
            println!();
            println!("注意: 请将 'username' 替换为实际的用户名");
        }
        Err(e) => {
            println!("✗ 哈希生成失败: {}", e);
        }
    }
}

fn batch_generate_hashes() {
    println!("批量生成常用密码哈希...\n");

    let common_passwords = vec![
        ("admin", "管理员默认密码"),
        ("admin123", "管理员常用密码"),
        ("password", "通用密码"),
        ("123456", "简单密码"),
        ("12345678", "8位数字密码"),
        ("admin2026", "管理员2026"),
        ("password123", "密码+数字"),
        ("manager", "经理默认密码"),
        ("manager123", "经理常用密码"),
        ("user", "普通用户默认密码"),
        ("user123", "普通用户常用密码"),
    ];

    println!("| {:<15} | {:<20} | 哈希值 |", "密码", "描述");
    println!("|{}|{}|{}|", "-".repeat(15), "-".repeat(20), "-".repeat(60));
    println!();

    for (password, description) in common_passwords {
        match password::hash_password(password) {
            Ok(hash) => {
                println!("| {:<15} | {:<20} | {} |", password, description, hash);
            }
            Err(e) => {
                println!("| {:<15} | {:<20} | ✗ 失败: {} |", password, description, e);
            }
        }
    }

    println!();
    println!("注意: 这些哈希值可以直接用于 SQL UPDATE 语句");
}

#[allow(dead_code)]
fn interactive_mode() {
    println!("交互式密码生成模式");
    println!("输入 'exit' 或 'quit' 退出\n");

    loop {
        print!("请输入密码: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("读取输入失败");

        let input = input.trim();

        if input == "exit" || input == "quit" {
            println!("退出...");
            break;
        }

        if input.is_empty() {
            println!("密码不能为空\n");
            continue;
        }

        match password::hash_password(input) {
            Ok(hash) => {
                println!("哈希值: {}\n", hash);
            }
            Err(e) => {
                println!("错误: {}\n", e);
            }
        }
    }
}

// 简化的密码哈希函数(独立版本,用于工具)
// 实际使用时应该导入 My_server/src/utils/password.rs 模块
#[allow(dead_code)]
mod password_simple {
    use argon2::password_hash::{PasswordHasher, SaltString};
    use argon2::Argon2;

    pub fn hash_password_simple(password: &str) -> Result<String, String> {
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut rand::thread_rng());

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| e.to_string())
    }
}
