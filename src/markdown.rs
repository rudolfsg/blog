use chrono::format::format;
use itertools::Itertools;
use lazy_static::lazy_static;
use pulldown_cmark::{html as pdc_html, CowStr, Event, LinkType, Options, Parser, Tag};
use std::{error, fs, io};
use tera::{Context, Tera};

use chrono::{Datelike, NaiveDate};
use katex;

fn parse_equation(text: &String) -> String {
    if text.starts_with("$$") && text.ends_with("$$") {
        if text.len() <= 4 {
            println!("Invalid display mode equation, will skip: {}", text);
            return text.clone();
        }
        // display mode equations
        let slice = &text[2..text.len() - 2];
        let is_valid = slice.find('$').is_none();
        if !is_valid {
            panic!("Invalid display (full) mode equation: {text}")
        }
        let opts = katex::Opts::builder().display_mode(true).build().unwrap();
        let equation =
            katex::render_with_opts(slice, &opts).expect("rendered display mode equation");
        return equation;
    } else {
        // inline equations
        let matches: Vec<_> = text.match_indices("$").collect();
        let mut indices = Vec::new();

        if matches.len() <= 1 {
            return text.clone()
        } else {
            // ignore dollar signs that are escaped

            for (i, _) in matches.iter() {
                if i > &0 && text.chars().nth(i - 1).unwrap() == '\\' {
                    continue;
                } else {
                    indices.push(i);
                }
            }
            if indices.len() % 2 != 0 {
                panic!("Parsing went wrong for equation: {text}")
            }

            let mut output = String::new();
            let mut previous_index = 0;
            loop {
                if indices.is_empty() {
                    break;
                }
                let start = indices[0];
                let end = indices[1];
                let mut slice = "";

                if *start == 0 {
                    slice = "";
                } else if previous_index == 0 {
                    slice = &text[..*start];
                } else {
                    slice = &text[previous_index + 1..*start];
                }
                output.push_str(slice);

                let equation = &text[start + 1..*end];

                let opts = katex::Opts::builder().display_mode(false).build().unwrap();
                let equation =
                    katex::render_with_opts(equation, &opts).expect("rendered inline equation");
                output.push_str(&equation);
                previous_index = *end;

                indices.remove(1);
                indices.remove(0);
            }
            if previous_index + 1 < text.len() {
                output.push_str(&text[previous_index + 1..])
            }
            return output;
        }
    }
}

pub fn parse_markdown(markdown: &String) -> String {

    let parser = Parser::new_ext(markdown, Options::all()).map(|event| 
        // {
        match &event {
            Event::Start(tag) => match tag {
             
                Tag::Image(link_type, url, title) => {
                    // read images from sibling instead of child folder
                    let new_url: CowStr = url.replace("images/", "..images/").into();
                    Event::Start(Tag::Image(link_type.to_owned(), new_url,title.to_owned() ))
                },
                _ => event,
            },
            Event::Text(text) => {
                if !text.contains('$') {
                    return event
                }
                else{
                    let equation = parse_equation(&text.to_string());
                    return Event::Html(equation.into())
                }

                }
                 _ => event,
            }
    );
           let mut html_output = String::new();
    pdc_html::push_html(&mut html_output, parser);
    return html_output
        }


