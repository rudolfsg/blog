use chrono::format::format;
use itertools::Itertools;
use lazy_static::lazy_static;
use pulldown_cmark::{
    html as pdc_html, CodeBlockKind, CowStr, Event, LinkType, Options, Parser, Tag,
};
use std::{error, fs, io, sync::Mutex};
use tera::{Context, Tera};

use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

use chrono::{Datelike, NaiveDate};
use katex;

lazy_static! {
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
}

fn parse_equation(text: &String) -> String {
    if text.len() <= 2 {
        text.clone()
    } else if text.starts_with("$$") && text.ends_with("$$") {
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
        equation
    } else {
        // inline equations
        let matches: Vec<_> = text.match_indices("$").collect();
        let mut indices = Vec::new();

        if matches.len() <= 1 {
            text.clone()
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
            output
        }
    }
}

fn highlight_code(code: &String, language: Option<String>) -> String {

    let syntax = match language {
        Some(s) => SYNTAX_SET.find_syntax_by_token(&s),
        None => None,
    };

    let syntax = match syntax {
        Some(s) => s,
        None => match SYNTAX_SET.find_syntax_by_first_line(&code) {
            Some(s) => {
                println!("Code syntax determined from lines, consider adding annotation");
                s
            }
            None => {
                println!("Failed to find code syntax");
                SYNTAX_SET.find_syntax_plain_text()
            }
        },
    };
    let html =
        highlighted_html_for_string(&code, &SYNTAX_SET, syntax, &THEME_SET.themes["base16-eighties.dark"]).expect("parsed codeblock");
    
        // drop the included background color 
    let start = html.find('>').expect("background color style") + 1; 
    let end = html.find("</pre>").expect("background color style"); 
    let html = &html[start..end].trim();
    let html = format!(r##"<pre><code class="code-block">{}</code></pre>"##, html); 
    return html
}

#[derive(Eq, PartialEq, Debug)]
enum MultiLineType {
    CodeBlock,
    DisplayModeMath,
    None,
}

pub fn parse_markdown(markdown: &String) -> String {
    let mut events = Vec::new();
    let mut multiline_type = MultiLineType::None;
    let mut buffer = String::new();

    for event in Parser::new_ext(markdown, Options::all()) {
        match event {
            Event::Start(Tag::Image(link_type, url, title)) => {
                // read images from sibling instead of child folder
                let new_url: CowStr = url.replace("images/", "/images/").into();
                let new_event =
                    Event::Start(Tag::Image(link_type.to_owned(), new_url, title.to_owned()));
                events.push(new_event);
            }
            Event::Start(Tag::CodeBlock(block)) => {
                buffer = String::new();
                multiline_type = MultiLineType::CodeBlock;
            }
            Event::End(Tag::CodeBlock(kind)) => {
                let language = match kind {
                    CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                    CodeBlockKind::Indented => None,
                };
                let html = highlight_code(&buffer, language); 
                events.push(Event::Html(CowStr::from(html)));
                multiline_type = MultiLineType::None;
            }

            Event::Text(text) => match multiline_type {
                MultiLineType::CodeBlock => buffer.push_str(&text),
                MultiLineType::DisplayModeMath => {
                    buffer.push_str(&text.trim().to_string());

                    if text.trim() == "$$" {
                        // end of multiline equation
                        let equation = parse_equation(&buffer);
                        events.push(Event::Html(equation.into()));
                        multiline_type = MultiLineType::None;
                    }
                }
                MultiLineType::None => {
                    if !text.contains('$') {
                        events.push(Event::Text(text));
                    } else if text.trim() == "$$" {
                        // start of multi line equation
                        buffer = String::new();
                        multiline_type = MultiLineType::DisplayModeMath;
                        buffer.push_str(&text);
                    } else {
                        let equation = parse_equation(&text.trim().to_string());
                        events.push(Event::Html(equation.into()));
                    }
                }
            },
            Event::Code(s) => {
                // inline code
                // synectic fails to find from first line so kinda useless now
                // let html = highlight_code(&s.to_string(), None); 
                // events.push(Event::Html(CowStr::from(html)));
                events.push(Event::Code(s))
            },
            _ => {
                // println!("{:?}", event);
                events.push(event);
            }
        }
    }

    if multiline_type != MultiLineType::None {
        println!(
            "Parsing failing due to multiline content: {:?}",
            multiline_type
        );
        panic!()
    }

    let mut html_output = String::new();
    pdc_html::push_html(&mut html_output, events.into_iter());
    return html_output;
}

// pub fn parse_markdown(markdown: &String) -> String {

//     let parser = Parser::new_ext(markdown, Options::all()).map(|event|
//         // {
//         match &event {
//             Event::Start(tag) => match tag {

//                 Tag::Image(link_type, url, title) => {
//                     // read images from sibling instead of child folder
//                     let new_url: CowStr = url.replace("images/", "/images/").into();
//                     Event::Start(Tag::Image(link_type.to_owned(), new_url,title.to_owned() ))
//                 },
//                 Tag::CodeBlock(block) => {
//                     println!("{:?}", block);
//                     // bloc.
//                     event
//                 }
//                 _ => event,
//             },
//             Event::Text(text) => {
//                 if !text.contains('$') {
//                     return event
//                 }
//                 else{
//                     let equation = parse_equation(&text.to_string());
//                     return Event::Html(equation.into())
//                 }

//                 }
//                  _ => event,
//             }
//     );
//     let mut html_output = String::new();
//     pdc_html::push_html(&mut html_output, parser);
//     return html_output
// }
