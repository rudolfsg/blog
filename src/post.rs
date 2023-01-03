use std::{error::Error, collections::HashMap};
use serde::{Serialize, Deserialize};
use chrono::NaiveDate;
use slug::slugify;

use crate::markdown; 

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Metadata{
    pub title: String, 
    pub date: NaiveDate, 
    pub slug: String, 
    pub tags: Vec<String>,
}
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
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

        let mut metadata: serde_yaml::Value = serde_yaml::from_str(metadata).unwrap();
        metadata["slug"] = slugify(metadata["title"].as_str().unwrap()).into();

        let metadata: Metadata = serde_yaml::from_value(metadata)?;
        // println!("{:?}", metadata); 
        Ok((metadata, contents.to_owned()))
    }
    pub fn from_string(contents: String) -> Result<Post, Box<dyn Error>>  {
        let (metadata, contents) = Post::extract_metadata(contents)?; 

        Ok(Post{metadata, contents})
    }
    pub fn render(&self) -> (String, bool, HashMap<String, f64>) {
        markdown::parse_markdown(&self.contents)
    }
}