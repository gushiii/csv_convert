use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::collections::HashMap;
use std::error::Error;
use std::io::{BufRead, BufReader, Cursor};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

/// 辅助函数: 计算居中矩形
/// percent_x: 水平方向占比（0-100）
/// percent_y: 垂直方向占比（0-100）
/// r: 可用区域
/// 返回一个新的 Rect，表示居中且占比指定的区域
pub fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    use ratatui::layout::{Constraint, Direction, Layout};
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// CSV 转换函数
/// input_path: 输入文件路径
/// target_ext: 目标文件扩展名
/// 返回转换后的字符串
pub fn convert_csv(input_path: &Path, target_ext: &str) -> Result<String, Box<dyn Error>> {
    let file = std::fs::File::open(input_path)?;
    let reader = BufReader::new(file);
    let mut cleaned_csv = String::new();

    for line in reader.lines() {
        let line = line?;
        let cleaned_line = line
            .split(',')
            .map(|s| s.trim())
            .collect::<Vec<_>>()
            .join(",");
        cleaned_csv.push_str(&cleaned_line);
        cleaned_csv.push('\n');
    }

    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .quoting(true)
        .double_quote(true)
        .from_reader(Cursor::new(cleaned_csv));

    let mut data = Vec::new();

    let headers: Vec<String> = reader
        .headers()?
        .iter()
        .map(|h| h.trim().to_string())
        .collect();

    for result in reader.records() {
        let record = result?;
        let mut map = HashMap::new();
        for (header, field) in headers.iter().zip(record.iter()) {
            map.insert(header.to_string(), field.to_string());
        }
        data.push(map);
    }

    let typed_data: Vec<HashMap<String, serde_json::Value>> = data
        .into_iter()
        .map(|map| {
            map.into_iter()
                .map(|(k, v)| {
                    let trimmed = v.trim();

                    let json_val = if trimmed.is_empty() {
                        serde_json::Value::Null
                    } else if let Ok(n) = trimmed.parse::<i64>() {
                        serde_json::json!(n)
                    } else if let Ok(f) = trimmed.parse::<f64>() {
                        serde_json::json!(f)
                    } else if let Ok(b) = trimmed.parse::<bool>() {
                        serde_json::json!(b)
                    } else {
                        serde_json::Value::String(trimmed.to_string())
                    };

                    (k, json_val)
                })
                .collect()
        })
        .collect();

    match target_ext {
        "json" => Ok(serde_json::to_string_pretty(&typed_data)?),
        "yaml" | "yml" => Ok(serde_yaml::to_string(&typed_data)?),
        "toml" => {
            let wrapped = serde_json::json!({ "rows": typed_data });
            Ok(toml::to_string_pretty(&wrapped)?)
        }
        _ => Err("Unsupported format".into()),
    }
}

/// 执行转换并返回结果
/// dest: 目标文件路径
/// src: 源文件路径
/// format: 目标格式（json、yaml、toml）
pub fn try_save(dest: &PathBuf, src: &PathBuf, format: &str) -> Result<(), String> {
    let content = convert_csv(src, format).map_err(|e| format!("解析失败: {}", e))?;
    if let Some(parent) = Path::new(dest).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建文件夹失败: {}", e))?;
    }
    std::fs::write(dest, content).map_err(|e| format!("写入失败: {}", e))?;

    Ok(())
}

/// 辅助函数: 将十六进制颜色字符串转换为 ratatui 的 Color
/// hex: 形如 "#RRGGBB" 的字符串
/// 示例: hex_to_color("#FF0000") -> Color::Rgb(255, 0, 0)
pub fn hex_to_color(hex: &str) -> Color {
    // 移除可能的 # 前缀
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Color::White; // 格式错误时返回默认色
    }

    // 将十六进制字符串解析为 RGB 数值
    if let Ok(rgb) = u32::from_str_radix(hex, 16) {
        let r = ((rgb >> 16) & 0xFF) as u8;
        let g = ((rgb >> 8) & 0xFF) as u8;
        let b = (rgb & 0xFF) as u8;
        Color::Rgb(r, g, b)
    } else {
        Color::White
    }
}

/// 辅助函数: 格式化路径，使用 ~ 代表 home 目录
/// path: 需要格式化的路径
/// 示例: 如果 home 目录是 /Users/alice，那么 /Users/alice/Documents 会被格式化为 ~/Documents
pub fn format_path(path: &std::path::Path) -> String {
    let path_str = path.to_string_lossy();
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        if path_str.starts_with(&*home_str) {
            return path_str.replacen(&*home_str, "~", 1);
        }
    }
    path_str.into_owned()
}

/// 将 syntect 的高亮颜色值转换为 ratatui Color
fn syntect_color_to_ratatui(color: syntect::highlighting::Color) -> Color {
    Color::Rgb(color.r, color.g, color.b)
}

/// 全局语法集合缓存
fn get_syntax_set() -> &'static SyntaxSet {
    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

/// 全局主题集合缓存
fn get_theme_set() -> &'static ThemeSet {
    static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();
    THEME_SET.get_or_init(ThemeSet::load_defaults)
}

/// 限制大小读取文件内容
/// path: 文件路径
/// max_size: 最大读取字节数
/// 返回读取的文件内容，如果超过大小则截断
pub fn read_file_limited(path: &Path, max_size: u64) -> Result<String, Box<dyn Error>> {
    use memmap2::MmapOptions;
    use std::fs::File;

    let file = File::open(path)?;
    let file_size = file.metadata()?.len();
    let read_size = std::cmp::min(file_size, max_size);

    // 对于小文件，使用传统读取
    if read_size <= 64 * 1024 {
        // 预分配并清空缓冲区，确保内存被正确管理
        let mut buffer = Vec::with_capacity(read_size as usize);
        buffer.resize(read_size as usize, 0);
        buffer.clear(); // 清空以确保干净的状态
        buffer.resize(read_size as usize, 0);

        let mut file = File::open(path)?;
        use std::io::Read;
        file.read_exact(&mut buffer)?;

        let result = validate_and_convert_to_string(&buffer);

        // 读取完成后清空缓冲区，释放内存
        buffer.clear();
        drop(buffer);

        return result;
    }

    // 对于大文件，使用内存映射
    let mmap = unsafe {
        MmapOptions::new()
            .len(read_size as usize)
            .map(&file)?
    };

    let result = validate_and_convert_to_string(&mmap);

    // 内存映射会在作用域结束时自动释放
    drop(mmap);

    result
}

/// 验证数据并转换为UTF-8字符串
/// 改进的二进制文件检测：检查控制字符比例和可打印字符比例
fn validate_and_convert_to_string(data: &[u8]) -> Result<String, Box<dyn Error>> {
    // 快速检查：包含空字节的一定是二进制文件
    if data.iter().any(|&b| b == 0) {
        return Err("(可能是二进制文件)".into());
    }

    // 计算可打印字符比例
    let printable_count = data.iter().filter(|&&b| {
        b.is_ascii_alphanumeric() || b.is_ascii_whitespace() || 
        b.is_ascii_punctuation() || b == b'\n' || b == b'\r' || b == b'\t'
    }).count();

    let printable_ratio = printable_count as f64 / data.len() as f64;

    // 如果可打印字符比例太低，可能是二进制文件
    if printable_ratio < 0.8 && data.len() > 100 {
        return Err("(可能是二进制文件)".into());
    }

    // 尝试转换为UTF-8
    String::from_utf8(data.to_vec())
        .map_err(|_| "文件包含无效的 UTF-8 字符".into())
}


/// 对源代码进行语法高亮
/// content: 源代码内容
/// file_path: 文件路径（用于推断语言）
/// 返回 ratatui Line 向量
pub fn highlight_code(content: &str, file_path: &Path) -> Vec<Line<'static>> {
    // 特殊处理CSV文件：渲染为表格格式
    if let Some(ext) = file_path.extension() {
        if ext == "csv" {
            return render_csv_as_table(content);
        }
    }

    let ps = get_syntax_set();
    let ts = get_theme_set();
    let theme = &ts.themes["Solarized (dark)"];

    // 根据文件扩展名推断语言
    let syntax = ps
        .find_syntax_for_file(file_path)
        .ok()
        .flatten()
        .or_else(|| {
            // 如果无法推断，尝试根据扩展名简单匹配
            let ext = file_path.extension()?;
            let ext_str = ext.to_str()?;
            ps.find_syntax_by_extension(ext_str)
        })
        .unwrap_or_else(|| ps.find_syntax_plain_text());

    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut lines = Vec::with_capacity(content.lines().count());

    for line_content in LinesWithEndings::from(content) {
        let highlighted = match highlighter.highlight_line(line_content, ps) {
            Ok(regions) => {
                let spans: Vec<Span> = regions
                    .into_iter()
                    .map(|(style, text)| {
                        let color = syntect_color_to_ratatui(style.foreground);
                        Span::styled(
                            text.replace('\n', ""),
                            ratatui::style::Style::default().fg(color),
                        )
                    })
                    .collect();
                Line::from(spans)
            }
            Err(_) => {
                // 如果高亮失败，降级为无高亮
                Line::from(line_content.replace('\n', ""))
            }
        };
        lines.push(highlighted);
    }

    lines
}

/// 将CSV内容渲染为美观的表格格式
/// content: CSV文件内容
/// 返回 ratatui Line 向量，模拟表格显示
pub fn render_csv_as_table(content: &str) -> Vec<Line<'static>> {
    use std::cmp;

    // let file = std::fs::File::open(input_path)?;
    let reader = BufReader::new(content.as_bytes());
    let mut cleaned_csv = String::new();

    for line in reader.lines() {
        let line = line.unwrap_or_default();
        let cleaned_line = line
            .split(',')
            .map(|s| s.trim())
            .collect::<Vec<_>>()
            .join(",");
        cleaned_csv.push_str(&cleaned_line);
        cleaned_csv.push('\n');
    }

    let mut lines = Vec::new();

    // 尝试解析CSV内容
    // let mut reader = csv::ReaderBuilder::new()
    //     .has_headers(true)
    //     .from_reader(content.as_bytes());
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .quoting(true)
        .double_quote(true)
        .from_reader(Cursor::new(cleaned_csv));

    // 获取表头
    let headers = match reader.headers() {
        Ok(h) => h.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        Err(_) => {
            // 如果无法解析表头，回退到原始文本显示
            return content.lines().map(|line| Line::from(line.to_string())).collect();
        }
    };

    if headers.is_empty() {
        return vec![Line::from("CSV文件为空或格式错误")];
    }

    // 读取所有记录，限制最多显示50行
    let mut records = Vec::new();
    for (i, result) in reader.records().enumerate() {
        if i >= 50 { // 限制显示行数
            break;
        }
        if let Ok(record) = result {
            records.push(record.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        }
    }

    if records.is_empty() {
        return vec![Line::from("CSV文件无数据行")];
    }

    // 计算每列的最大宽度
    let mut column_widths = vec![0; headers.len()];
    for (i, header) in headers.iter().enumerate() {
        column_widths[i] = cmp::max(column_widths[i], header.len());
    }
    for record in &records {
        for (i, field) in record.iter().enumerate() {
            if i < column_widths.len() {
                column_widths[i] = cmp::max(column_widths[i], field.len());
            }
        }
    }

    // 限制列宽，避免表格过宽
    for width in &mut column_widths {
        *width = cmp::min(*width, 25); // 最大25个字符
    }

    // 创建表格边框样式
    let border_color = Color::Indexed(244); // 灰色
    let header_color = Color::Cyan;
    let data_color = Color::White;

    // 渲染表头
    let mut header_line = Vec::new();
    header_line.push(Span::styled("┌", border_color));
    for (i, (header, &width)) in headers.iter().zip(&column_widths).enumerate() {
        let padded_header = format!(" {:<width$} ", header, width = width);
        header_line.push(Span::styled(padded_header, Style::default().fg(header_color).add_modifier(Modifier::BOLD)));
        if i < headers.len() - 1 {
            header_line.push(Span::styled("│", border_color));
        }
    }
    header_line.push(Span::styled("┐", border_color));
    lines.push(Line::from(header_line));

    // 渲染分隔线
    let mut separator_line = Vec::new();
    separator_line.push(Span::styled("├", border_color));
    for (i, &width) in column_widths.iter().enumerate() {
        separator_line.push(Span::styled("─".repeat(width + 2), border_color));
        if i < column_widths.len() - 1 {
            separator_line.push(Span::styled("┼", border_color));
        }
    }
    separator_line.push(Span::styled("┤", border_color));
    lines.push(Line::from(separator_line));

    // 渲染数据行
    for (row_idx, record) in records.iter().enumerate() {
        let mut data_line = Vec::new();
        data_line.push(Span::styled("│", border_color));

        for (i, field) in record.iter().enumerate() {
            if i < column_widths.len() {
                let width = column_widths[i];
                // 截断过长的字段
                let display_field = if field.len() > width {
                    format!("{}…", &field[..width.saturating_sub(1)])
                } else {
                    field.clone()
                };
                let padded_field = format!(" {:<width$} ", display_field, width = width);
                data_line.push(Span::styled(padded_field, Style::default().fg(data_color)));
            }
            if i < record.len().saturating_sub(1) {
                data_line.push(Span::styled("│", border_color));
            }
        }

        data_line.push(Span::styled("│", border_color));
        lines.push(Line::from(data_line));

        // 每行添加分隔线（除最后一行外）
        if row_idx < records.len() - 1 {
            let mut mid_separator = Vec::new();
            mid_separator.push(Span::styled("├", border_color));
            for (i, &width) in column_widths.iter().enumerate() {
                mid_separator.push(Span::styled("─".repeat(width + 2), border_color));
                if i < column_widths.len() - 1 {
                    mid_separator.push(Span::styled("┼", border_color));
                }
            }
            mid_separator.push(Span::styled("┤", border_color));
            lines.push(Line::from(mid_separator));
        }
    }

    // 渲染表格底部
    let mut bottom_line = Vec::new();
    bottom_line.push(Span::styled("└", border_color));
    for (i, &width) in column_widths.iter().enumerate() {
        bottom_line.push(Span::styled("─".repeat(width + 2), border_color));
        if i < column_widths.len() - 1 {
            bottom_line.push(Span::styled("┴", border_color));
        }
    }
    bottom_line.push(Span::styled("┘", border_color));
    lines.push(Line::from(bottom_line));

    // 添加统计信息
    let total_rows = records.len();
    let total_cols = headers.len();
    lines.push(Line::from(""));
    lines.push(Line::from(format!("📊 表格信息: {} 行 × {} 列", total_rows, total_cols)));

    // 如果记录被截断，显示提示
    if records.len() >= 50 {
        lines.push(Line::from("⚠️  显示已限制为前50行"));
    }

    lines
}
