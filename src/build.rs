use crate::html::{TEMPLATES, minify_css, minify_html};
use crate::image_convert::{self, modify_url};
use crate::{BUILD_DIR, MINIFY};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::collections::HashMap;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::{fs, io};
use tera::Context;

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
        fs::remove_dir_all("build").expect("deleted build dir");
    }

    create_folder(BUILD_DIR);
    create_folder(&format!("{BUILD_DIR}/images"));
    create_folder(&format!("{BUILD_DIR}/posts"));
    create_folder(&format!("{BUILD_DIR}/tags"));
    create_folder(&format!("{BUILD_DIR}/fonts"));
}


pub fn build_html(name: &str, template_name: &str, path: &str, context: Context) {
    let mut html = TEMPLATES
        .render(template_name, &context)
        .expect(&format!("{template_name} render"));

    if MINIFY {
        html = minify_html(&html);
    }
    let mut path: String = path.to_string();
    if name != "index" {
        path = format!("{}{}{}/", BUILD_DIR, path, name);
        create_folder(&path);
    } else {
        path = format!("{}/", BUILD_DIR);
    }

    let path = PathBuf::from(format!("{}index.html", path));
    std::fs::write(path, html).expect(&format!("{template_name} html write"));
}

pub fn copy_assets(source: &str, dest: &str) {
    let folder = fs::read_dir(source).unwrap();
    for file in folder {
        let file_path = file.unwrap().path();
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        if file_name.starts_with('.') {
            continue;
        } else if file_path.is_dir() {
            copy_assets(
                PathBuf::from(source).join(file_name).to_str().unwrap(),
                PathBuf::from(dest).join(file_name).to_str().unwrap(),
            );
            continue;
        };

        let path = PathBuf::from(dest).join(file_name);

        if file_path.extension().unwrap() == "css" && MINIFY {
            // minify css
            let mut css = fs::read_to_string(file_path).unwrap();
            css = minify_css(&css);
            fs::write(path, css).expect("css write");
        } else {
            fs::copy(file_path, path).expect("copy file");
        }
    }
}

fn process_single_image(file: &DirEntry, dest: &str, scaling: &f64) -> bool {
    let file_path = file.path();
    let file_name = file_path.file_name().unwrap().to_str().unwrap().to_string();
    if file_name.starts_with('.') || file_path.is_dir() {
        return false;
    }

    let file_name = modify_url(file_name);
    let dest = PathBuf::from(format!("{}{}{}", BUILD_DIR, dest, file_name));

    image_convert::convert_image(file_path, dest, *scaling);
    true
}

pub fn process_images(source: &str, dest: &str, image_scales: &HashMap<String, f64>) {
    let files: Vec<DirEntry> = fs::read_dir(source)
        .unwrap()
        .map(|result| result.unwrap())
        .collect();

    let _result: Vec<bool> = files
        .par_iter()
        .map(|f| {
            process_single_image(
                f,
                dest,
                image_scales.get(&f.path().file_name().unwrap().to_str().unwrap().to_string()).unwrap_or(&1.0),
            )
        })
        .collect();
}