use std::{fs, path::Path};

use resvg::{tiny_skia, usvg};

fn svg2bitmap<P: AsRef<Path>>(svg: P) -> (Vec<u8>, u32, u32) {
    let svg_tree = usvg::Tree::from_str(
        fs::read_to_string(svg).unwrap().as_str(),
        &Default::default(),
    )
    .unwrap();

    let (w, h) = {
        let size = svg_tree.size();
        (size.width() as u32, size.height() as u32)
    };

    let mut pixmap = tiny_skia::Pixmap::new(w, h).unwrap();

    resvg::render(&svg_tree, Default::default(), &mut pixmap.as_mut());

    (pixmap.data().to_vec(), w, h)
}

fn main() {
    let (data, w, h) = svg2bitmap("assets/logo_v.svg");
    println!("Bitmap size: {}x{}, data length: {}", w, h, data.len());

    let image_buf = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(w, h, data).unwrap();

    image_buf.save("output_logo.png").unwrap();
}
