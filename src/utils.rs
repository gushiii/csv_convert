use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use ratatui::style::Color;

// 辅助函数: 计算居中矩形
pub fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
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

pub fn convert_csv(input_path: &Path, target_ext: &str) -> Result<String, Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(input_path)?;
    let mut data = Vec::new();
    let headers = reader.headers()?.clone();

    for result in reader.records() {
        let record = result?;
        let mut map = HashMap::new();
        for (header, field) in headers.iter().zip(record.iter()) {
            map.insert(header.to_string(), field.to_string());
        }
        data.push(map);
    }

    match target_ext {
        "json" => Ok(serde_json::to_string_pretty(&data)?),
        "yaml" | "yml" => Ok(serde_yaml::to_string(&data)?),
        "toml" => {
            let wrapped = serde_json::json!({ "rows": data });
            Ok(toml::to_string_pretty(&wrapped)?)
        }
        _ => Err("Unsupported format".into()),
    }
}

// 执行转换并返回结果
pub fn try_save(dest: &PathBuf, src: &PathBuf, format: &str) -> Result<(), String> {
    let content = convert_csv(src, format).map_err(|e| format!("解析失败: {}", e))?;
    if let Some(parent) = Path::new(dest).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建文件夹失败: {}", e))?;
    }
    std::fs::write(dest, content).map_err(|e| format!("写入失败: {}", e))?;

    Ok(())
}

// 辅助函数: 将十六进制颜色字符串转换为 ratatui 的 Color
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

// 辅助函数: 格式化路径，使用 ~ 代表 home 目录
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