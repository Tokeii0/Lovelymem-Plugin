# Lovelymem-Plugin

一个专为内存取证分析设计的高性能工具集，包含两个核心组件：字符串提取工具和路径转换器。

## 🚀 项目概述

本项目提供了两个强大的内存取证工具：

- **memstrap** - 高性能CLI内存取证字符串提取工具
- **openMemprocfsPath** - memprocfs路径转换器，用于在Windows资源管理器中快速定位文件

## 📁 项目结构

```
Lovelymem-Plugin/
├── memstrap/                    # 内存字符串提取工具
│   ├── src/                     # 源代码
│   ├── examples/                # 使用示例
│   ├── tests/                   # 测试文件
│   ├── Cargo.toml              # 项目配置
│   └── README.md               # 详细文档
├── openMemprocfsPath/          # 路径转换工具
│   ├── src/                    # 源代码
│   ├── Cargo.toml             # 项目配置
│   └── README.md              # 详细文档
└── README.md                  # 项目总览（本文件）
```

## 🛠️ 快速开始

### 环境要求

- Rust 1.70+
- Windows 操作系统（openMemprocfsPath工具需要）
- 足够的内存处理大型内存镜像文件

### 编译安装

```bash
# 克隆项目
git clone <repository-url>
cd Lovelymem-Plugin

# 编译 memstrap
cd memstrap
cargo build --release

# 编译 openMemprocfsPath
cd ../openMemprocfsPath
cargo build --release
```

编译完成后，可执行文件位于各自的 `target/release/` 目录中。

## 🔧 工具介绍

### 1. memstrap - 内存字符串提取工具

一个专为内存取证设计的高性能字符串提取工具，支持多种编码格式和并行处理。

#### 主要特性

- ⚡ **高性能**: 多线程处理，支持GB到TB级别的内存镜像
- 🔍 **多编码支持**: ASCII、UTF-8、UTF-16LE、UTF-16BE
- 🎯 **智能搜索**: 支持纯文本和正则表达式搜索
- 📊 **结构化输出**: CSV格式输出，便于后续分析
- 🚀 **内存映射**: 高效的I/O操作，无需将整个文件加载到内存

#### 快速使用

```bash
# 基本用法 - 提取所有字符串
./memstrap/target/release/memstrap memory_dump.raw

# 搜索特定模式
./memstrap/target/release/memstrap memory_dump.raw -s "password" -o results.csv

# 使用正则表达式查找邮箱地址
./memstrap/target/release/memstrap memory_dump.raw -s "\w+@\w+\.\w+" -r
```

### 2. openMemprocfsPath - 路径转换工具

将memprocfs虚拟路径转换为本地文件系统路径，并在Windows资源管理器中快速定位文件。

#### 主要特性

- 🔄 **路径转换**: 支持多种memprocfs路径格式
- 📂 **自动打开**: 在Windows资源管理器中自动定位文件
- 🎯 **多模式支持**: normal、ntfs、vol2三种转换模式
- ⚠️ **错误处理**: 详细的错误信息和状态反馈

#### 快速使用

```bash
# 转换并打开ntfs路径
./openMemprocfsPath/target/release/open_memprocfs_path.exe "\0\Windows\System32\config\SYSTEM"

# 转换files路径（normal模式）
./openMemprocfsPath/target/release/open_memprocfs_path.exe "\0\Windows\System32\config\SYSTEM" normal

# 转换卷路径（vol2模式）
./openMemprocfsPath/target/release/open_memprocfs_path.exe "\Device\HarddiskVolume2\Windows\System32" vol2
```

## 📖 详细文档

每个工具都有详细的使用文档：

- [memstrap 详细文档](./memstrap/README.md)
- [openMemprocfsPath 详细文档](./openMemprocfsPath/README.md)

## 🧪 测试

```bash
# 测试 memstrap
cd memstrap
cargo test

# 测试 openMemprocfsPath
cd openMemprocfsPath
cargo test
```

## 🤝 使用场景

### 内存取证分析
- 从内存转储中提取敏感信息（密码、邮箱、URL等）
- 分析恶意软件留下的字符串痕迹
- 数字取证调查中的证据收集

### 文件系统分析
- 快速定位memprocfs中的文件和目录
- 在取证分析过程中提高工作效率
- 批量处理文件路径转换

## 📝 输出格式

### memstrap输出格式
CSV格式，包含以下字段：
- FilePath: 输入文件路径
- Offset(Hex): 十六进制偏移量
- Offset(Dec): 十进制偏移量
- Encoding: 检测到的编码格式
- Length: 字符串字节长度
- Content: 提取的字符串内容

### openMemprocfsPath输出
- 显示原路径和转换后路径
- 自动在Windows资源管理器中打开目标位置
- 提供详细的状态信息和错误提示

## ⚡ 性能优化

- **内存映射**: 避免将大文件完全加载到内存
- **并行处理**: 充分利用多核CPU性能
- **块重叠**: 防止字符串在边界处被分割
- **进度显示**: 大文件处理时显示实时进度

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🔗 相关链接

- [memprocfs项目](https://github.com/ufrisk/MemProcFS)
- [Rust官方文档](https://doc.rust-lang.org/)
- [内存取证最佳实践](https://www.volatilityfoundation.org/)

---

**注意**: 本工具仅用于合法的数字取证和安全研究目的。使用时请遵守相关法律法规。