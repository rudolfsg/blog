use image::*;
use std::path::PathBuf;
use webp::*;
use std::fs;


const CONVERT_TO_WEBP: &[&str] = &["jpg", "jpeg", "png"];

pub fn modify_url(url: String) -> String {
    let mut new_url: String = url.replace("images/", "/images/");
    let ext = new_url.rfind('.').unwrap();

    if CONVERT_TO_WEBP.contains(&&new_url[ext + 1..]) {
        new_url = format!("{}.webp", &new_url[..ext]);
    }

    new_url
}

pub fn get_image_dims(url: &str) -> Result<imagesize::ImageSize, imagesize::ImageError> {
    let dims = imagesize::size(format!("posts/{}", url));
    dims
}

pub fn convert_image(source: PathBuf, dest: PathBuf, size_factor: f64) {
    let ext = source.extension().unwrap();
    let ext = ext.to_str().unwrap();

    if CONVERT_TO_WEBP.contains(&ext) {
        let mut img = image::open(&source).unwrap();
        let (w, h) = img.dimensions();
        if size_factor != 1.0 {
            println!("Rescaling {:?}", &source);
            img = image::DynamicImage::ImageRgba8(imageops::resize(
                &img,
                (w as f64 * size_factor) as u32,
                (h as f64 * size_factor) as u32,
                imageops::FilterType::Gaussian,
            ));
        }
        let encoder: Encoder = Encoder::from_image(&img).unwrap();
        let webp: WebPMemory;
        if ext == "png" {
            webp = encoder.encode_lossless();
        }
        else {
            webp = encoder.encode(85f32)
        };
        std::fs::write(&dest, &*webp).unwrap();
    } else {
        if size_factor != 1.0 {
            println!("Rescaling not supported for {ext}, skipping {source:?}")
        }
        fs::copy(source, dest).expect("copy file");

    }

}
