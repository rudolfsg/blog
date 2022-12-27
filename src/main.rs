#![allow(unused)]

use chrono::format::format;
use lazy_static::lazy_static;
use std::{error, fs, io};
use chrono::{Datelike, NaiveDate};
use tera::{Context, Tera};
use std::path::{Path, PathBuf};


mod post;
mod markdown; 
mod html;
use post::Post;

// TODO:
// posts in correct date order
// process html in parallel



fn create_folder(path: &str) {
    match fs::create_dir(path) {
        Ok(_) => (),
        Err(err) => match err.kind() {
            io::ErrorKind::AlreadyExists => (),
            _ => panic!("Could not create folder {}", path),
        },
    }
}


fn main() {
    create_folder("build");
    create_folder("build/images");
    create_folder("build/posts");

    // build posts
    let markdown_files = fs::read_dir("posts").unwrap();
    let mut posts: Vec<Post> = Vec::new();

    const BLOG_NAME: &str = "My Blog";

    for file in markdown_files {
        let file_path = file.unwrap().path();
        let file_name = file_path.file_name().unwrap().to_str().unwrap();

        if file_name.starts_with(".") || file_path.is_dir() {
            continue;
        } // ignore .DS_Store etc
        let file_ext = file_path.extension().unwrap().to_str().unwrap();
        if file_ext != "md" {
            println!("Skipping non-markdown file: {}", file_path.display());
            continue;
        }
        println!("Processing: {}", file_name);

        let markdown_input = fs::read_to_string(file_path).unwrap();

        let post = Post::from_string(markdown_input).unwrap();

        let output = post.render();

        let mut context = Context::new();
        context.insert("title", &post.metadata.title);
        context.insert("content", &output);
        context.insert("date", &post.metadata.date);
        context.insert("tags", &post.metadata.tags);
        let output = html::TEMPLATES
            .render("post.html", &context)
            .expect("post rendering");

        let path = format!("build/posts/{}.html", &post.metadata.slug);
        let path = PathBuf::from(path);
        std::fs::write(path, output).expect("post html writing");

        posts.push(post);

    }
    // create index.html
    let index_content = html::create_index(&mut posts);
    let mut context = Context::new();
    context.insert("content", &index_content);
    context.insert("title", &BLOG_NAME);
    let output = html::TEMPLATES
        .render("index.html", &context)
        .expect("index rendering");

    let path = PathBuf::from("build/index.html");
    std::fs::write(path, output).expect("index html writing");

    // copy images
    let images = fs::read_dir("posts/images").unwrap();
    for file in images {
        let file_path = file.unwrap().path();
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        if file_name.starts_with(".") || file_path.is_dir() {
            continue;
        }

        let dest = PathBuf::from(format!("build/images/{}", file_name));
        fs::copy(file_path, dest).expect("copy image");
    }

    println!("Done");
  
}
