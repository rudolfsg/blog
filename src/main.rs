use std::fs;
use itertools::Itertools;
use rayon::prelude::*; 
use tera::Context;
use std::collections::HashMap;
use slug::slugify;
use fs_extra::dir::get_size;

mod post;
mod markdown; 
mod html;
mod build; 
mod image_convert;

use post::Post;

// TODO:
// consider SASS instead of plain CSS for nesting, variable defs etc
// light theme toggle

pub const BUILD_DIR: &str = "build";
pub const MINIFY: bool = false;
const BLOG_NAME: &str = "blog";
const CLEAN_BUILD: bool = true; 

fn main() {

    use std::time::Instant;
    let time = Instant::now();

    

    build::init_build(CLEAN_BUILD); 

    // build posts
    let markdown_files = fs::read_dir("posts").unwrap().collect_vec();
    let mut posts: Vec<Post> = Vec::new();
    // todo: merge all scales after for loop
    let mut image_scale : HashMap<String, f64> = HashMap::new();

    for file in markdown_files {
        let file_path = file.unwrap().path();
        let file_name = file_path.file_name().unwrap().to_str().unwrap();

        if file_name.starts_with('.') || file_path.is_dir() {
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

        let (output, has_katex, post_image_scale) = post.render();

        // might be duplicated - warn user
        image_scale.extend(post_image_scale);

        let mut context = Context::new();
        context.insert("title", &post.metadata.title);
        context.insert("content", &output);
        context.insert("has_katex", &has_katex);
        context.insert("date", &post.metadata.date);
        context.insert("tags", &post.metadata.tags);

        build::build_html(&post.metadata.slug, "post.html", "/posts/", context);

        posts.push(post);

    }
    // create index.html
    let index_content = html::create_index(& posts);
    let mut context = Context::new();
    context.insert("content", &index_content);
    context.insert("title", &BLOG_NAME);
    build::build_html("index", "index.html", "/", context);

    // about
    let mut context = Context::new();
    context.insert("title", &BLOG_NAME);
    build::build_html("about", "about.html", "/", context);

    // create tag indices
    let mut unique_tags: HashMap<&str, Vec<Post>> = HashMap::new();
    for post in &posts{
        for tag in &post.metadata.tags{
            if unique_tags.contains_key(tag as &str) {
                let v: &mut Vec<Post> = unique_tags.get_mut(tag as &str).unwrap();
                v.push(post.clone());
            } 
            else {
                let v: Vec<Post> = vec![post.clone()]; 
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
        build::build_html(&slugify(tag), "index.html", "/tags/", context);

        all_tags.push(tag);

    }

    // all tags index
    all_tags.sort();
    let mut context = Context::new();
        context.insert("tags", &all_tags);
        context.insert("title", "All tags");
        build::build_html("all-tags", "all-tags.html", "/tags/", context);
 

    build::copy_assets("assets", BUILD_DIR);
    build::process_images("posts/images","/images/" , &image_scale);


    let elapsed = time.elapsed();
    println!("Done in: {:.2?}", elapsed);

    let folder_size = get_size(BUILD_DIR).unwrap() / 1024;
    println!("Build size: {}KB", folder_size); // print directory sile in bytes
  
}
