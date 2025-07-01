# Memprocfs Path Converter

这是一个 Rust 命令行工具，用于将 memprocfs 路径转换为本地文件系统路径，并在 Windows 资源管理器中打开并选中对应的文件或文件夹。

## 功能

- 将形如 `\0\Windows\System32\config\SYSTEM` 的路径转换为 `M:\forensic\ntfs\0\Windows\System32\config\SYSTEM`
- 自动在 Windows 资源管理器中打开并选中转换后的文件/文件夹
- 提供详细的错误信息和帮助文档

## 编译

```bash
cd plugin/openMemprocfsPath
cargo build --release
```

编译完成后，可执行文件将位于 `target/release/open_memprocfs_path.exe`

## 使用方法

### 基本用法

```bash
./target/release/open_memprocfs_path.exe "\0\Windows\System32\config\SYSTEM"
```

### 显示帮助

```bash
./target/release/open_memprocfs_path.exe --help
```

## 示例

### 输入
```
\0\Windows\System32\config\SYSTEM
```

### 输出
```
原路径: \0\Windows\System32\config\SYSTEM
转换后: M:\forensic\ntfs\0\Windows\System32\config\SYSTEM
成功在资源管理器中打开并选中文件！
```

程序会自动打开 Windows 资源管理器并选中 `M:\forensic\ntfs\0\Windows\System32\config\SYSTEM` 文件。

## 错误处理

- 如果目标路径不存在，程序会显示警告但仍然尝试打开资源管理器
- 如果无法启动资源管理器，程序会显示详细的错误信息
- 如果缺少参数，程序会显示使用帮助

## 测试

```bash
cargo test
```

运行单元测试以验证路径转换功能是否正常工作。 