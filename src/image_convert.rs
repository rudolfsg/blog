use image::*;
use imagesize;
use std::path::{Path, PathBuf};
use webp::*;

pub fn modify_url(url: String) -> String {
    let new_url: String = url.replace("images/", "/images/");
    let ext = new_url.rfind('.').unwrap();
    let new_url = format!("{}.webp", &new_url[..ext]);
    new_url
}

pub fn get_image_dims(url: &str) -> Result<imagesize::ImageSize, imagesize::ImageError>{
    let dims = imagesize::size(format!("posts/{}", url));
    dims
}

pub fn convert_image(source: PathBuf, dest: PathBuf, size_factor: f64) {
    // Using `image` crate, open the included .jpg file
    let img = image::open(source).unwrap();
    let (w, h) = img.dimensions();
    // Optionally, resize the existing photo and convert back into DynamicImage
    let img: DynamicImage = image::DynamicImage::ImageRgba8(imageops::resize(
        &img,
        (w as f64 * size_factor) as u32,
        (h as f64 * size_factor) as u32,
        imageops::FilterType::Triangle,
    ));

    // Create the WebP encoder for the above image
    let encoder: Encoder = Encoder::from_image(&img).unwrap();
    // Encode the image at a specified quality 0-100
    let webp: WebPMemory = encoder.encode(75f32);
    // Define and write the WebP-encoded file to a given path
    std::fs::write(&dest, &*webp).unwrap();
}
