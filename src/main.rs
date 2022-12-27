use pulldown_cmark::{html, Options, Parser};
use std::fs;
use std::path::Path;

mod post; 
use post::Post; 
use katex;
// TODO: 
// posts in correct date order
// process html in parallel
fn main() {
    let markdown_files = fs::read_dir("posts").unwrap();

    for file in markdown_files {
        let file_path = file.unwrap().path();
        let file_name = file_path.file_name().unwrap().to_str().unwrap();
        let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
        let markdown_input = fs::read_to_string(file_path).unwrap();

        let post = Post::from_string(markdown_input).unwrap(); 

        let output = post.render(); 

        println!("{}", output);

        let html = katex::render("E = mc^2").unwrap();

let opts = katex::Opts::builder().display_mode(true).build().unwrap();
let html_in_display_mode = katex::render_with_opts("E = mc^2", &opts).unwrap();
        println!("{}", html_in_display_mode);


        // // Set up options and parser. Strikethroughs are not part of the CommonMark standard
        // // and we therefore must enable it explicitly.
        // let mut options = Options::empty();
        // options.insert(Options::ENABLE_STRIKETHROUGH);
        // let parser = Parser::new_ext(&markdown_input, options);

        // // Write to String buffer.
        // let mut html_output = String::new();
        // html::push_html(&mut html_output, parser);

        // println!("{}", html_output);
    }

    println!("Done");
}
