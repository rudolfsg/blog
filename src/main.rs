use fs_extra::dir::get_size;
use rayon::prelude::*;
use slug::slugify;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tera::Context;

mod build;
mod html;
mod image_convert;
mod markdown;
mod post;
mod rss;

use post::Post;

pub const BUILD_DIR: &str = "build";
pub const MINIFY: bool = true;
const BLOG_NAME: &str = "blog";
const BLOG_URL: &str = "https://blog.stuff.com"; 
const CLEAN_BUILD: bool = true;

fn process_post(path: &PathBuf) -> (Post, HashMap<String, f64>) {
        let file_name = path.file_name().unwrap().to_str().unwrap();

        
        println!("Processing: {}", file_name);

        let markdown_input = fs::read_to_string(path).unwrap();

        let post = Post::from_string(markdown_input).unwrap();

        let (output, has_katex, post_image_scale) = post.render();

        let mut context = Context::new();
        context.insert("title", &post.metadata.title);
        context.insert("content", &output);
        context.insert("has_katex", &has_katex);
        context.insert("date", &post.metadata.date);
        context.insert("tags", &post.metadata.tags);
        context.insert("url", &BLOG_URL); 

        build::build_html(&post.metadata.slug, "post.html", "/posts/", context);

        (post, post_image_scale)
}

fn main() {
    use std::time::Instant;
    let time = Instant::now();
        
    let mut posts: Vec<Post> = Vec::new();
    let mut image_scale: HashMap<String, f64> = HashMap::new();

    build::init_build(CLEAN_BUILD);
     // build posts
    let markdown_files: Vec<PathBuf> = fs::read_dir("posts")
        .unwrap()
        .map(|result| result.unwrap().path())
        .filter(|p| p.is_file() && p.extension().unwrap_or_else(|| std::ffi::OsStr::new("invalid")) == "md")
        .collect();

    let result: Vec<(Post, HashMap<String, f64>)> = markdown_files
        .par_iter()
        .map(process_post)
        .collect();

    for (post, scale) in result.into_iter() {
        posts.push(post);
        image_scale.extend(scale);
    }

    // create index.html
    let index_content = html::create_index(&posts);
    let mut context = Context::new();
    context.insert("content", &index_content);
    context.insert("title", &BLOG_NAME);
    context.insert("url", &BLOG_URL); 
    build::build_html("index", "index.html", "/", context);

    // about
    let mut context = Context::new();
    context.insert("title", &BLOG_NAME);
    context.insert("url", &BLOG_URL); 

    build::build_html("about", "about.html", "/", context);

    // create tag indices
    let mut unique_tags: HashMap<&str, Vec<Post>> = HashMap::new();
    for post in &posts {
        for tag in &post.metadata.tags {
            if unique_tags.contains_key(tag as &str) {
                let v: &mut Vec<Post> = unique_tags.get_mut(tag as &str).unwrap();
                v.push(post.clone());
            } else {
                let v: Vec<Post> = vec![post.clone()];
                unique_tags.insert(tag, v);
            }
        }
    }
    let mut all_tags = Vec::new();
    for (tag, tag_posts) in unique_tags {
        let index_content = html::create_index(&tag_posts);
        let mut context = Context::new();
        context.insert("content", &index_content);
        context.insert("title", &tag);
        context.insert("index_title", &tag);
        context.insert("url", &BLOG_URL); 

        build::build_html(&slugify(tag), "index.html", "/tags/", context);

        all_tags.push(tag);
    }

    // all tags index
    all_tags.sort();
    let mut context = Context::new();
    context.insert("tags", &all_tags);
    context.insert("title", "All tags");
    context.insert("url", &BLOG_URL); 
    build::build_html("all-tags", "all-tags.html", "/tags/", context);
    // rss
    let rss_xml = rss::generate_rss(&posts, BLOG_NAME, BLOG_URL); 
    fs::write(format!("{}/rss.xml", BUILD_DIR), rss_xml).unwrap(); 
    // assets
    build::copy_assets("assets", BUILD_DIR);
    build::process_images("posts/images", "/images/", &image_scale);

    let elapsed = time.elapsed();
    println!("Done in: {:.2?}", elapsed);

    let folder_size = get_size(BUILD_DIR).unwrap() / 1024;
    println!("Build size: {}KB", folder_size);
}
