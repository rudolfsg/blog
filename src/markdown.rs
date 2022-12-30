use chrono::format::format;
use itertools::{Itertools, MultiPeek};
use std::collections::HashMap;
use std::path::PathBuf;
use lazy_static::lazy_static;
use pulldown_cmark::{
    html as pdc_html, CodeBlockKind, CowStr, Event, LinkType, Options, Parser, Tag,
};
use std::{error, fs, io, sync::Mutex};
use tera::{Context, Tera};

use chrono::{Datelike, NaiveDate};
use katex;

use crate::html;
use crate::image_convert;

pub struct EventIterator<'a, I: Iterator<Item = Event<'a>>> {
    parser: MultiPeek<I>,
    has_katex: bool,
    image_scale: HashMap<String, f64>,
}

impl<'a, I: Iterator<Item = Event<'a>>> EventIterator<'a, I> {
    pub fn new(parser: I) -> Self {
        Self {
            parser: parser.multipeek(),
            has_katex: false,
            image_scale: HashMap::new(),
        }
    }

    pub fn enable_katex(&mut self) {
        self.has_katex = true;
    }

    pub fn add_image(&mut self, url: String, scaling: f64) {
        self.image_scale.insert(url, scaling);
    }
}

impl<'a, I: Iterator<Item = Event<'a>>> Iterator for EventIterator<'a, I> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.parser.next() {
            match event {
                // images + figures
                Event::Start(Tag::Image(link_type, url, title)) => {
                    
                    let caption = match &self.parser.next() {
                        Some(Event::Text(t)) => {
                            self.parser.next();
                            Some(t.to_owned().to_string())
                        },
                        _ => None,
                    };
                    // end of figure
                    // self.parser.next();

                    // extract width=x%.
                    let scaling = match &self.parser.peek() {
                        Some(Event::Text(t)) => {
                            let s = t.replace(' ', "");
                            let pattern = "width=";
                            if s.starts_with('{') && s.ends_with('}') && s.contains(pattern) {
                                let start = s.find('=').expect("width specified with =") + 1;
                                let end = s.find('%').expect("width specified with %");
                                let s = &s[start..end];
                                let scaling: usize = s.parse().expect("number conversion");
                                let scaling = (scaling as f64) / 100.0;
                                self.parser.next();

                                let key = PathBuf::from(url.to_string()).file_name().unwrap().to_str().unwrap().to_string(); 
                                self.add_image(key, scaling);

                                Some(scaling)
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    let html = html::create_figure(url.clone().to_string(), caption, scaling);

                    return Some(Event::Html(html.into()));
                }
                // code blocks
                Event::Start(Tag::CodeBlock(block)) => {
                    let mut buffer = String::new();

                    loop {
                        let next = self.parser.next();
                        if let Some(Event::Text(text)) = next {
                            buffer.push_str(&text);
                        } else if let Some(Event::End(Tag::CodeBlock(kind))) = next {
                            let language = match kind {
                                CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                                CodeBlockKind::Indented => None,
                            };
                            let html = html::highlight_code(&buffer, language);

                            return Some(Event::Html(CowStr::from(html)));
                        } else {
                            panic!("Code block end was not received")
                        }
                    }
                }
                Event::Text(text) => {
                    if !text.contains('$') {
                        // no equation
                        Some(Event::Text(text))
                    } 
                    else if text.trim() == "$$" {
                        // multi line equation
                        let mut buffer = "$$".to_string();
                        loop {
                            let next = self.parser.next();
                            match next {
                                Some(e) => match e {
                                    Event::Text(text) => {
                                        buffer.push_str(&text);
                                        if text.trim() == "$$" {
                                            let (equation, flag) = html::parse_equation(&buffer);
                                            if flag {self.enable_katex()}
                                            return Some(Event::Html(equation.into()))
                                        }
                                    }
                                    _ => (),
                                },
                                None => panic!("Multi line equation wasnt terminated"),
                            }
                        }
                    } 
                    else {
                        let (equation, flag) = html::parse_equation(&text.trim().to_string());
                        if flag {self.enable_katex()}
                        Some(Event::Html(equation.into()))
                    }
                }
                _ => Some(event),
            }
        } else {
            return None;
        }
    }
}

pub fn parse_markdown(markdown: &String) -> (String, bool, HashMap<String, f64>) {
    let parser = Parser::new_ext(markdown, Options::all());
    let mut iterator = EventIterator::new(parser);
    let mut html = String::new();

    pdc_html::push_html(&mut html, &mut iterator);
    let has_katex = iterator.has_katex;
    let image_scale = iterator.image_scale.clone();

    (html, has_katex, image_scale)
}

// #[derive(Eq, PartialEq, Debug)]
// enum MultiLineType {
//     CodeBlock,
//     DisplayModeMath,
//     None,
// }

// pub fn parse_markdown(markdown: &String) -> (String, bool) {
//     let mut events = Vec::new();
//     let mut multiline_type = MultiLineType::None;
//     let mut buffer = String::new();
//     let mut has_katex = false;

//     for event in Parser::new_ext(markdown, Options::all()) {
//         match event {
//             Event::Start(Tag::Image(link_type, url, title)) => {
//                 // read images from sibling instead of child folder
//                 let new_url: CowStr = url.replace("images/", "/images/").into();
//                 let new_event =
//                     Event::Start(Tag::Image(link_type.to_owned(), new_url, title.to_owned()));
//                 events.push(new_event);
//             }
//             Event::Start(Tag::CodeBlock(block)) => {
//                 buffer = String::new();
//                 multiline_type = MultiLineType::CodeBlock;
//             }
//             Event::End(Tag::CodeBlock(kind)) => {
//                 let language = match kind {
//                     CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
//                     CodeBlockKind::Indented => None,
//                 };
//                 let html = highlight_code(&buffer, language);
//                 events.push(Event::Html(CowStr::from(html)));
//                 multiline_type = MultiLineType::None;
//             }

//             Event::Text(text) => match multiline_type {
//                 MultiLineType::CodeBlock => buffer.push_str(&text),
//                 MultiLineType::DisplayModeMath => {
//                     buffer.push_str(&text.trim().to_string());

//                     if text.trim() == "$$" {
//                         // end of multiline equation
//                         let (equation, flag) = parse_equation(&buffer);
//                         events.push(Event::Html(equation.into()));
//                         multiline_type = MultiLineType::None;
//                         if flag {
//                             has_katex = true
//                         };
//                     }
//                 }
//                 MultiLineType::None => {
//                     if !text.contains('$') {
//                         events.push(Event::Text(text));
//                     } else if text.trim() == "$$" {
//                         // start of multi line equation
//                         buffer = String::new();
//                         multiline_type = MultiLineType::DisplayModeMath;
//                         buffer.push_str(&text);
//                     } else {
//                         let (equation, flag) = parse_equation(&text.trim().to_string());
//                         events.push(Event::Html(equation.into()));
//                         if flag {
//                             has_katex = true
//                         };
//                     }
//                 }
//             },
//             Event::Code(s) => {
//                 // inline code
//                 // synectic fails to find from first line so kinda useless now
//                 // let html = highlight_code(&s.to_string(), None);
//                 // events.push(Event::Html(CowStr::from(html)));
//                 events.push(Event::Code(s))
//             }
//             _ => {
//                 // println!("{:?}", event);
//                 events.push(event);
//             }
//         }
//     }

//     if multiline_type != MultiLineType::None {
//         println!(
//             "Parsing failing due to multiline content: {:?}",
//             multiline_type
//         );
//         panic!()
//     }

//     let mut html_output = String::new();
//     pdc_html::push_html(&mut html_output, events.into_iter());
//     return (html_output, has_katex);
// }
