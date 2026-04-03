# CSV Convert

[![Rust](https://img.shields.io/badge/rust-1.56%2B-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一个基于Rust的终端用户界面(TUI)应用程序，用于在CSV向JSON、YAML和TOML格式进行数据转换和处理。提供直观的交互式界面，支持文件浏览、数据预览和格式转换。

## ✨ 功能特性

- **多格式支持**: 支持CSV转换为JSON、YAML、TOML格式
- **交互式TUI界面**: 基于Ratatui的现代化终端用户界面
- **文件浏览器**: 内置文件浏览器，方便选择输入/输出文件
- **实时预览**: 支持数据预览和语法高亮
- **键盘驱动**: 完全键盘操作，无需鼠标
- **跨平台**: 支持macOS、Linux和Windows
- **高效处理**: 支持大文件处理，使用内存映射优化性能

## 🛠 技术栈

- **Rust** - 系统编程语言，提供内存安全和高性能
- **Ratatui** - 现代Rust TUI框架
- **Crossterm** - 跨平台终端处理库
- **Serde** - Rust序列化框架，支持多种数据格式
- **Syntect** - 语法高亮库

## 📁 项目结构

```
csv_convert/
├── src/
│   ├── main.rs      # 应用入口和事件循环
│   ├── app.rs       # 应用逻辑和状态管理
│   ├── ui.rs        # UI渲染逻辑
│   └── utils.rs     # 工具函数和数据处理
├── assets/
│   ├── input.csv    # CSV格式示例数据
│   ├── input.json   # JSON格式示例数据
│   ├── input.toml   # TOML格式示例数据
│   └── input.yaml   # YAML格式示例数据
├── Cargo.toml       # 项目依赖配置
├── LICENSE          # MIT许可证
└── README.md        # 项目文档
```

## 🚀 安装与运行

### 前置要求

- Rust 1.56 或更高版本
- 支持Unicode的终端 (推荐使用Nerd Font字体以获得最佳视觉效果)

### 安装Rust

如果尚未安装Rust，请访问 [rustup.rs](https://rustup.rs/) 进行安装：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 构建项目

```bash
# 克隆仓库
git clone https://github.com/gushiii/csv_convert.git
cd csv_convert

# 构建发布版本
cargo build --release
```

### 运行应用

```bash
# 直接运行
cargo run

# 或运行已构建的二进制文件
./target/release/csv_convert
```

## 📖 使用方法

### 基本操作

启动应用后，您将看到一个分栏界面：

1. **文件浏览器** (左侧): 浏览和选择输入文件
2. **数据预览** (右侧): 显示选中文件的内容和语法高亮
3. **输出面板** (弹出层): 显示转换结果

### 键盘快捷键

- `↑/↓` - 在文件列表中导航
- `[Enter]/F` - 选择文件或确认操作
- `Q` - 退出应用
- `J/K` - 滚动显示文件预览

## 📋 示例

项目包含示例数据文件在 `assets/` 目录下：

### CSV 示例 (input.csv)
```csv
Name,Position,DOB,Nationality,KitNumber
Wojciech Szczesny,Goalkeeper,"Apr 18, 1990 (29)",Poland,1
Mattia Perin,Goalkeeper,"Nov 10, 1992 (26)",Italy,37
```

### JSON 示例 (input.json)
```json
[
  {
    "Name": "Wojciech Szczesny",
    "Position": "Goalkeeper",
    "DOB": "Apr 18,1990 (29)",
    "Nationality": "Poland",
    "KitNumber": 1
  }
]
```

### YAML 示例 (input.yaml)
```yaml
- Name: Wojciech Szczesny
  Position: Goalkeeper
  DOB: "Apr 18,1990 (29)"
  Nationality: Poland
  KitNumber: 1
```

### TOML 示例 (input.toml)
```toml
[[players]]
Name = "Wojciech Szczesny"
Position = "Goalkeeper"
DOB = "Apr 18,1990 (29)"
Nationality = "Poland"
KitNumber = 1
```

### 开发环境设置

```bash
# 安装开发依赖
cargo install cargo-edit cargo-watch

# 运行开发服务器 (自动重编译)
cargo watch -x run
```

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 👤 作者

**gushiii** - [zrjie001@gmail.com](mailto:zrjie001@gmail.com)

## 🙏 致谢

- [Ratatui](https://github.com/ratatui-org/ratatui) - 优秀的Rust TUI框架
- [Serde](https://github.com/serde-rs/serde) - Rust序列化框架
- [Crossterm](https://github.com/crossterm-rs/crossterm) - 终端处理库

---

⭐ 如果这个项目对你有帮助，请给它一个星标！
