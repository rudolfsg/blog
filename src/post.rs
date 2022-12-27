use std::{error::Error};
use serde::{Serialize, Deserialize};
use chrono::{NaiveDate, Datelike} ;
use pulldown_cmark::{html, Options, Parser};

use crate::markdown; 

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Metadata{
    pub title: String, 
    pub date: NaiveDate, 
    pub slug: String, 
    pub tags: Vec<String>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Post {
    pub metadata: Metadata,
    pub contents: String, 
}


impl Post {
    pub fn extract_metadata(contents: String) -> Result<(Metadata, String), Box<dyn Error>> {
        if !contents.starts_with("---\n"){
            return Err("invalid metadata")?
        }
        let slice = &contents[4..]; 
        let metadata_end = slice.find("---\n").expect("metadata delimiter check");
        let metadata = &slice[..metadata_end];  
        let contents = &slice[metadata_end..]; 

        let metadata: Metadata = serde_yaml::from_str(&metadata)?;
        // println!("{:?}", metadata); 
        Ok((metadata, contents.to_owned()))
    }
    pub fn from_string(contents: String) -> Result<Post, Box<dyn Error>>  {
        let (metadata, contents) = Post::extract_metadata(contents)?; 

        Ok(Post{metadata, contents})
    }
    pub fn render(&self) -> String {
        // Set up options and parser. Strikethroughs are not part of the CommonMark standard
        // and we therefore must enable it explicitly.
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        let parser = Parser::new_ext(&self.contents, options);

        // Write to String buffer.
        let mut html_output = markdown::parse_markdown(&self.contents);
        html_output
    }
}