#![allow(unused)]

use chrono::format::format;
use lazy_static::lazy_static;
use std::{error, fs, io};
use chrono::{Datelike, NaiveDate};
use tera::{Context, Tera};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use slug::slugify;


mod post;
mod markdown; 
mod html;
use post::Post;

// TODO:
// posts in correct date order
// process html in parallel
// figures for images - resizing?
// use https://developer.mozilla.org/en-US/docs/Web/HTML/Element/dl for index


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

    use std::time::Instant;
    let time = Instant::now();

    create_folder("build");
    create_folder("build/images");
    create_folder("build/posts");
    create_folder("build/tags");


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
    let index_content = html::create_index(& posts);
    let mut context = Context::new();
    context.insert("content", &index_content);
    context.insert("title", &BLOG_NAME);
    let output = html::TEMPLATES
        .render("index.html", &context)
        .expect("index rendering");

    let path = PathBuf::from("build/index.html");
    std::fs::write(path, output).expect("index html writing");

    // about
    let mut context = Context::new();
    context.insert("title", &BLOG_NAME);

    let output = html::TEMPLATES
        .render("about.html", &context)
        .expect("about rendering");
    let path = PathBuf::from("build/about.html");
    std::fs::write(path, output).expect("about html writing");
    
    // create tag indices
    let mut unique_tags: HashMap<&str, Vec<Post>> = HashMap::new();
    for post in &posts{
        for tag in &post.metadata.tags{
            if unique_tags.contains_key(tag as &str) {
                let v: &mut Vec<Post> = unique_tags.get_mut(tag as &str).unwrap();
                v.push(post.clone());
            } 
            else {
                let mut v: Vec<Post> = Vec::new();
                v.push(post.clone()); 
                unique_tags.insert(tag, v); 
            }
        }
    }
    let mut all_tags = Vec::new();
    for (tag, tag_posts) in unique_tags {
        let index_content = html::create_index(& tag_posts);
        let mut context = Context::new();
        context.insert("content", &index_content);
        context.insert("title", &tag);
        context.insert("index_title", &tag);
        let output = html::TEMPLATES
            .render("index.html", &context)
            .expect("tag index rendering");
        let path = PathBuf::from(format!("build/tags/{}.html", slugify(tag)));
        std::fs::write(path, output).expect("index html writing");

        all_tags.push(tag);

    }

    // all tags index
    all_tags.sort();
    let mut context = Context::new();
        context.insert("tags", &all_tags);
        context.insert("title", "All tags");
        let output = html::TEMPLATES
            .render("all-tags.html", &context)
            .expect("tag index rendering");

        let path = PathBuf::from("build/tags/all-tags.html");
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

    // copy assets
    let images = fs::read_dir("assets").unwrap();
    for file in images {
        let file_path = file.unwrap().path();
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        if file_name.starts_with(".") || file_path.is_dir() {
            continue;
        }

        let dest = PathBuf::from(format!("build/{}", file_name));
        fs::copy(file_path, dest).expect("copy asset");
    }

    let elapsed = time.elapsed();
    println!("Done in: {:.2?}", elapsed);
  
}
