
use tera::{Context, Tera};
use lazy_static::lazy_static;
use crate::post::Post;
use itertools::Itertools;
use chrono::{Datelike, NaiveDate};

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html"]);
        tera
    };
}

pub fn create_index(posts: &mut Vec<Post>) -> String {
    posts.sort_by_key(|p| p.metadata.date);
    posts.reverse();
    let mut index_content = "".to_string();
    for (year, year_posts) in posts
        .iter()
        .group_by(|x| x.metadata.date.year().to_string())
        .into_iter()
    {
        index_content.push_str(&format!("<h3> {} </h3> <ul>", year));
        for post in year_posts {
            let link = format!(
                r##"<li><a href="posts/{}.html">{}</a></li>"##,
                post.metadata.slug, post.metadata.title
            );
            index_content.push_str(&link);
        }
        index_content.push_str("</ul>")
    }
    index_content
}