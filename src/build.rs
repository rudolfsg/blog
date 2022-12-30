use crate::html::TEMPLATES;
use crate::image_convert::{modify_url, self};
use minify_html::{minify, Cfg};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{error, fs, io};
use tera::Context;

pub const BUILD_DIR: &str = "build";
const MINIFY: bool = false; 


pub fn create_folder(path: &str) {
    match fs::create_dir(path) {
        Ok(_) => (),
        Err(err) => match err.kind() {
            io::ErrorKind::AlreadyExists => (),
            _ => panic!("Could not create folder {}", path),
        },
    }
}

pub fn init_build(clean_build: bool) {
    if clean_build {
        fs::remove_dir_all("build");
    }

    create_folder(&format!("{BUILD_DIR}"));
    create_folder(&format!("{BUILD_DIR}/images"));
    create_folder(&format!("{BUILD_DIR}/posts"));
    create_folder(&format!("{BUILD_DIR}/tags"));
}

fn minify_html(html: &String) -> String {
    let mut cfg = Cfg::spec_compliant();
        let minified = minify(html.as_bytes(), &cfg);
        let minified = String::from_utf8(minified).expect("convert type"); 
    minified
}

pub fn build_html(name: &str, template_name: &str, path: &str, context: Context) {
    let mut html = TEMPLATES
        .render(&template_name, &context)
        .expect(&format!("{template_name} render"));

    if MINIFY {
        html = minify_html(&html);
    }
    

    let path = format!("{}{}{}.html", BUILD_DIR, path, name);
    let path = PathBuf::from(path);
    std::fs::write(path, html).expect(&format!("{template_name} html write"));
}

pub fn copy_assets(source: &str, dest: &str) {
    let folder = fs::read_dir(source).unwrap();
    for file in folder {
        let file_path = file.unwrap().path();
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        if file_name.starts_with(".") || file_path.is_dir() {
            continue;
        }

        let dest = PathBuf::from(format!("{}{}{}", BUILD_DIR, dest, file_name));

        if file_path.extension().unwrap() == "css" && MINIFY {
            // minify css
            let mut css = fs::read_to_string(file_path).unwrap();
            css = minify_html(&css); 
            fs::write(dest, css).expect("css write"); 
        }
        else {
             
            fs::copy(file_path, dest).expect("copy file");
        }

       
    }
}

pub fn process_images(source: &str, dest: &str, image_scales: HashMap<String, f64>) {
    let folder = fs::read_dir(source).unwrap();

    for file in folder {
        let file_path = file.unwrap().path();
        let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();
        if file_name.starts_with(".") || file_path.is_dir() {
            continue;
        }

        let scaling = match image_scales.get(&file_name) {
            Some(s) => s.clone(),
            None => 1.0
        };

        let file_name = modify_url(file_name); 
        let dest = PathBuf::from(format!("{}{}{}", BUILD_DIR, dest, file_name));

        image_convert::convert_image(file_path, dest, scaling);
}
}
