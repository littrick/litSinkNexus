use anyhow::Context;
use resvg::{usvg, tiny_skia};

/// 支持更多选项的SVG转换
pub struct SvgToBitmapOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub background_color: Option<[u8; 4]>, // RGBA
    pub keep_aspect_ratio: bool,
    pub fit_type: FitType,
}

pub enum FitType {
    Contain,  // 保持纵横比，完全显示
    Cover,    // 保持纵横比，覆盖整个区域
    Fill,     // 拉伸填充
}

/// 高级转换函数
pub fn svg_to_bitmap(
    svg_data: &str,
    options: &SvgToBitmapOptions,
) -> anyhow::Result<(Vec<u8>, u32, u32)> {
    let tree = usvg::Tree::from_str(svg_data, &Default::default())?;
    
    // 确定目标尺寸
    let (target_width, target_height) = match (options.width, options.height) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => {
            let aspect = tree.size().height() / tree.size().width();
            (w, (w as f32 * aspect) as u32)
        }
        (None, Some(h)) => {
            let aspect = tree.size().width() / tree.size().height();
            ((h as f32 * aspect) as u32, h)
        }
        (None, None) => {
            let size = tree.size();
            (size.width() as u32, size.height() as u32)
        }
    };
    
    let mut pixmap = tiny_skia::Pixmap::new(target_width, target_height)
        .context("无法创建像素图")?;
    
    // 设置背景色
    if let Some(bg_color) = options.background_color {
        let paint = tiny_skia::Paint {
            shader: tiny_skia::Shader::SolidColor(tiny_skia::Color::from_rgba8(
                bg_color[0], bg_color[1], bg_color[2], bg_color[3],
            )),
            ..Default::default()
        };
        pixmap.fill(tiny_skia::Color::TRANSPARENT);
        let rect = tiny_skia::Rect::from_xywh(0.0, 0.0, 
            target_width as f32, target_height as f32).unwrap();
        pixmap.fill_rect(rect, &paint, tiny_skia::Transform::identity(), None);
    }
    
    // 计算变换矩阵
    let transform = match options.fit_type {
        FitType::Fill => {
            let scale_x = target_width as f32 / tree.size().width();
            let scale_y = target_height as f32 / tree.size().height();
            tiny_skia::Transform::from_scale(scale_x, scale_y)
        }
        FitType::Contain => {
            let scale_x = target_width as f32 / tree.size().width();
            let scale_y = target_height as f32 / tree.size().height();
            let scale = scale_x.min(scale_y);
            let dx = (target_width as f32 - tree.size().width() * scale) / 2.0;
            let dy = (target_height as f32 - tree.size().height() * scale) / 2.0;
            tiny_skia::Transform::from_translate(dx, dy).pre_scale(scale, scale)
        }
        FitType::Cover => {
            let scale_x = target_width as f32 / tree.size().width();
            let scale_y = target_height as f32 / tree.size().height();
            let scale = scale_x.max(scale_y);
            let dx = (target_width as f32 - tree.size().width() * scale) / 2.0;
            let dy = (target_height as f32 - tree.size().height() * scale) / 2.0;
            tiny_skia::Transform::from_translate(dx, dy).pre_scale(scale, scale)
        }
    };
    
    // 渲染
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    
    Ok((pixmap.data().to_vec(), target_width, target_height))
}

fn main() {
    // 示例用法
    let svg_data = r#"<svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
        <circle cx="50" cy="50" r="40" stroke="green" stroke-width="4" fill="yellow" />
    </svg>"#;

    let options = SvgToBitmapOptions {
        width: Some(200),
        height: Some(200),
        background_color: Some([255, 255, 255, 255]), // 白色背景
        keep_aspect_ratio: true,
        fit_type: FitType::Contain,
    };

    match svg_to_bitmap(svg_data, &options) {
        Ok((bitmap_data, width, height)) => {
            println!("转换成功，宽度: {}, 高度: {}, 数据长度: {}", width, height, bitmap_data.len());
            // 保存为bmp
            let mut file = "output.bmp";
            let bmp = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, bitmap_data).unwrap();
            bmp.save(&mut file).unwrap();
        }
        Err(e) => {
            eprintln!("转换失败: {}", e);
        }
    }
}