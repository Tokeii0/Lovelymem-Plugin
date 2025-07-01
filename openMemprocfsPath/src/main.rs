use std::env;
use std::process::Command;
use std::path::Path;

/// 将 memprocfs 路径转换为本地文件系统路径
fn convert_path(input_path: &str, mode: &str) -> (String, bool) {
    // 移除开头的反斜杠（如果存在）
    let cleaned_path = input_path.strip_prefix("\\").unwrap_or(input_path);
    
    let (new_path, is_directory) = match mode {
        "normal" => {
            // normal 模式：M:\forensic\files\ROOT\ + 目录部分
            if let Some(parent_pos) = cleaned_path.rfind('\\') {
                let directory_part = &cleaned_path[..parent_pos];
                let path = format!("M:\\forensic\\files\\ROOT\\{}", directory_part);
                (path, true) // 返回目录
            } else {
                // 如果没有路径分隔符，直接返回 ROOT 目录
                ("M:\\forensic\\files\\ROOT".to_string(), true)
            }
        },
        "ntfs" => {
            // ntfs 模式：M:\forensic\ntfs\ + 原路径
            let path = format!("M:\\forensic\\ntfs\\{}", cleaned_path);
            (path, false) // 返回文件
        },
        "vol2" => {
            // vol2 模式：解析 \Device\HarddiskVolumeX 格式
            if cleaned_path.starts_with("Device\\HarddiskVolume") {
                // 查找第三个反斜杠的位置（Device\HarddiskVolumeX\后面的部分）
                let parts: Vec<&str> = cleaned_path.splitn(3, '\\').collect();
                if parts.len() >= 3 {
                    // 提取卷号 (HarddiskVolumeX)
                    let volume_part = parts[1]; // HarddiskVolumeX
                    if volume_part.starts_with("HarddiskVolume") {
                        if let Ok(volume_num) = volume_part["HarddiskVolume".len()..].parse::<i32>() {
                            // 卷号减1作为目录索引
                            let target_num = volume_num - 1;
                            let remaining_path = parts[2]; // 剩余路径部分
                            let path = format!("M:\\forensic\\ntfs\\{}\\{}", target_num, remaining_path);
                            (path, false) // 返回文件
                        } else {
                            // 解析卷号失败，返回原路径
                            let path = format!("M:\\forensic\\ntfs\\{}", cleaned_path);
                            (path, false)
                        }
                    } else {
                        // 不是标准的HarddiskVolume格式
                        let path = format!("M:\\forensic\\ntfs\\{}", cleaned_path);
                        (path, false)
                    }
                } else {
                    // 路径格式不正确
                    let path = format!("M:\\forensic\\ntfs\\{}", cleaned_path);
                    (path, false)
                }
            } else {
                // 不以Device\HarddiskVolume开头，当作普通路径处理
                let path = format!("M:\\forensic\\ntfs\\{}", cleaned_path);
                (path, false)
            }
        },
        _ => {
            // 默认使用 ntfs 模式
            let path = format!("M:\\forensic\\ntfs\\{}", cleaned_path);
            (path, false)
        }
    };
    
    // 将正斜杠转换为反斜杠（Windows路径格式）
    (new_path.replace("/", "\\"), is_directory)
}

/// 在 Windows 资源管理器中打开并选中指定文件/文件夹
fn open_and_select_file(file_path: &str, is_directory: bool) -> Result<(), Box<dyn std::error::Error>> {
    // 检查路径是否存在
    let path_exists = Path::new(file_path).exists();
    
    if is_directory || !path_exists {
        // 如果是目录模式或路径不存在，直接打开目录
        let target_dir = if is_directory {
            file_path.to_string()
        } else if let Some(parent) = Path::new(file_path).parent() {
            parent.to_string_lossy().to_string()
        } else {
            file_path.to_string()
        };
        
        if !Path::new(&target_dir).exists() {
            eprintln!("警告: 目录不存在: {}", target_dir);
        }
        
        let output = Command::new("explorer.exe")
            .arg(&target_dir)
            .output()?;
        
        if output.status.success() {
            println!("成功打开目录: {}", target_dir);
            return Ok(());
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("无法打开目录 {}: {}", target_dir, error_msg).into());
        }
    }

    // 尝试选中具体文件
    println!("尝试选中文件: {}", file_path);
    
    // 方法1: 使用 /select 参数（单个参数）
    let select_arg = format!("/select,\"{}\"", file_path);
    let output = Command::new("explorer.exe")
        .arg(&select_arg)
        .output()?;

    if output.status.success() {
        println!("成功选中文件!");
        return Ok(());
    }

    // 方法2: 使用分离的参数
    let output = Command::new("explorer.exe")
        .arg("/select,")
        .arg(file_path)
        .output()?;

    if output.status.success() {
        println!("成功选中文件!");
        return Ok(());
    }

    // 方法3: 如果选中失败，直接打开父目录
    if let Some(parent) = Path::new(file_path).parent() {
        println!("选中失败，打开父目录: {}", parent.display());
        let fallback_output = Command::new("explorer.exe")
            .arg(parent.to_string_lossy().as_ref())
            .output()?;
            
        if fallback_output.status.success() {
            println!("成功打开父目录");
            return Ok(());
        }
    }
    
    Ok(())
}

/// 显示使用帮助
fn show_help() {
    println!("Memprocfs Path Converter");
    println!("用途: 将 memprocfs 路径转换为本地文件系统路径并在资源管理器中打开");
    println!();
    println!("使用方法:");
    println!("  {} <模式> <路径>", env::args().next().unwrap_or_else(|| "open_memprocfs_path".to_string()));
    println!();
    println!("模式:");
    println!("  ntfs     NTFS模式 - 将路径转换为 M:\\forensic\\ntfs\\<路径>");
    println!("  normal   Normal模式 - 将文件路径转换为对应目录 M:\\forensic\\files\\ROOT\\<目录>");
    println!("  vol2     Volatility2模式 - 将 \\Device\\HarddiskVolumeX 路径转换为对应的 ntfs 路径");
    println!();
    println!("示例:");
    println!("  {} ntfs \"\\0\\Windows\\System32\\config\\SYSTEM\"", 
             env::args().next().unwrap_or_else(|| "open_memprocfs_path".to_string()));
    println!("  将会转换为: M:\\forensic\\ntfs\\0\\Windows\\System32\\config\\SYSTEM");
    println!();
    println!("  {} normal \"\\Windows\\System32\\en-US\\KernelBase.dll.mui\"", 
             env::args().next().unwrap_or_else(|| "open_memprocfs_path".to_string()));
    println!("  将会转换为: M:\\forensic\\files\\ROOT\\Windows\\System32\\en-US\\");
    println!();
    println!("  {} vol2 \"\\Device\\HarddiskVolume1\\Windows\\System32\\wlanhlp.dll\"", 
             env::args().next().unwrap_or_else(|| "open_memprocfs_path".to_string()));
    println!("  将会转换为: M:\\forensic\\ntfs\\0\\Windows\\System32\\wlanhlp.dll");
    println!();
    println!("参数:");
    println!("  -h, --help    显示此帮助信息");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // 检查参数
    if args.len() < 2 {
        // eprintln!("错误: 缺少参数");
        show_help();
        std::process::exit(1);
    }

    let first_arg = &args[1];

    // 检查是否需要显示帮助
    if first_arg == "-h" || first_arg == "--help" {
        show_help();
        return;
    }

    // 检查是否有足够的参数（模式 + 路径）
    if args.len() < 3 {
        // eprintln!("错误: 缺少路径参数");
        show_help();
        std::process::exit(1);
    }

    let mode = &args[1];
    let input_path = &args[2];

    // 验证模式参数
    if mode != "ntfs" && mode != "normal" && mode != "vol2" {
        // eprintln!("错误: 无效的模式 '{}'，只支持 'ntfs'、'normal' 或 'vol2'", mode);
        show_help();
        std::process::exit(1);
    }

    // 转换路径
    let (converted_path, is_directory) = convert_path(input_path, mode);
    
    // println!("模式: {}", mode);
    // println!("原路径: {}", input_path);
    // println!("转换后: {}", converted_path);

    // 在资源管理器中打开并选中文件
    match open_and_select_file(&converted_path, is_directory) {
        Ok(()) => {
            if is_directory {
                println!("success");
            } else {
                println!("success");
            }
        }
        Err(_e) => {
            // eprintln!("错误: {}", _e);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_path_ntfs() {
        let (path, is_dir) = convert_path("\\0\\Windows\\System32\\config\\SYSTEM", "ntfs");
        assert_eq!(path, "M:\\forensic\\ntfs\\0\\Windows\\System32\\config\\SYSTEM");
        assert_eq!(is_dir, false);
        
        let (path, is_dir) = convert_path("0\\Windows\\System32\\config\\SYSTEM", "ntfs");
        assert_eq!(path, "M:\\forensic\\ntfs\\0\\Windows\\System32\\config\\SYSTEM");
        assert_eq!(is_dir, false);
        
        let (path, is_dir) = convert_path("\\0\\test\\file.txt", "ntfs");
        assert_eq!(path, "M:\\forensic\\ntfs\\0\\test\\file.txt");
        assert_eq!(is_dir, false);
    }

    #[test]
    fn test_convert_path_normal() {
        let (path, is_dir) = convert_path("\\Windows\\System32\\en-US\\KernelBase.dll.mui", "normal");
        assert_eq!(path, "M:\\forensic\\files\\ROOT\\Windows\\System32\\en-US");
        assert_eq!(is_dir, true);
        
        let (path, is_dir) = convert_path("Windows\\System32\\config\\SYSTEM", "normal");
        assert_eq!(path, "M:\\forensic\\files\\ROOT\\Windows\\System32\\config");
        assert_eq!(is_dir, true);
        
        let (path, is_dir) = convert_path("test.txt", "normal");
        assert_eq!(path, "M:\\forensic\\files\\ROOT");
        assert_eq!(is_dir, true);
    }

    #[test]
    fn test_convert_path_vol2() {
        // 测试标准的 Device\HarddiskVolumeX 格式
        let (path, is_dir) = convert_path("\\Device\\HarddiskVolume1\\Windows\\System32\\wlanhlp.dll", "vol2");
        assert_eq!(path, "M:\\forensic\\ntfs\\0\\Windows\\System32\\wlanhlp.dll");
        assert_eq!(is_dir, false);
        
        let (path, is_dir) = convert_path("Device\\HarddiskVolume2\\Windows\\notepad.exe", "vol2");
        assert_eq!(path, "M:\\forensic\\ntfs\\1\\Windows\\notepad.exe");
        assert_eq!(is_dir, false);
        
        let (path, is_dir) = convert_path("\\Device\\HarddiskVolume3\\Program Files\\test.dll", "vol2");
        assert_eq!(path, "M:\\forensic\\ntfs\\2\\Program Files\\test.dll");
        assert_eq!(is_dir, false);
        
        // 测试不标准的路径格式（应该当作普通ntfs路径处理）
        let (path, is_dir) = convert_path("\\SomeOther\\Path\\file.txt", "vol2");
        assert_eq!(path, "M:\\forensic\\ntfs\\SomeOther\\Path\\file.txt");
        assert_eq!(is_dir, false);
    }
} 