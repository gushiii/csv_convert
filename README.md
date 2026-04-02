# CSV Convert

一个基于Rust的终端用户界面(TUI)应用程序，用于转换和处理CSV文件。

## 功能特性

- 交互式终端用户界面
- CSV文件处理和转换
- 实时界面更新
- 键盘事件处理

## 技术栈

- **Rust** - 编程语言
- **ratatui** - 终端UI框架
- **crossterm** - 跨平台终端处理

## 项目结构

```tree
csv_convert/
├── src/
│   ├── main.rs      # 应用入口和事件循环
│   ├── app.rs       # 应用逻辑和状态管理
│   ├── ui.rs        # UI渲染逻辑
│   └── utils.rs     # 工具函数
├── Cargo.toml       # 项目依赖配置
└── README.md        # 项目文档
```

## 安装与运行

### 前置要求

- Rust 1.56+

### 构建项目

```bash
cargo build --release
```

### 运行应用

```bash
cargo run
```

## 使用方法

启动应用后，使用键盘进行交互操作。按相应的键盘快捷键执行相应功能。

## 开发

### 清理构建

```bash
cargo clean
```

### 运行测试

```bash
cargo test
```

## 许可证

MIT
