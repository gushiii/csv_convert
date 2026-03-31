use ratatui::style::Color;
use std::collections::HashMap;
use std::error::Error;
use std::io::{BufRead, BufReader, Cursor};
use std::path::{Path, PathBuf};

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
