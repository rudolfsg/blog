use crate::image_convert;
use crate::post::Post;
use chrono::Datelike;
use css_minify::optimizations::{Level, Minifier};
use itertools::Itertools;
use lazy_static::lazy_static;
use minify_html::{minify, Cfg};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use tera::Tera;

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
    static ref THEME_SET: ThemeSet = ThemeSet::load_defaults();
    static ref SYNTAX_SET: SyntaxSet = SyntaxSet::load_defaults_newlines();
}

pub fn minify_html(html: &String) -> String {
    let mut cfg = Cfg::spec_compliant();
    cfg.minify_css = true;
    let minified = minify(html.as_bytes(), &cfg);
    String::from_utf8(minified).expect("convert type")
}

pub fn minify_css(css: &String) -> String {
    // nb: minify_css crashes with double ;
    Minifier::default()
        .minify(&css.replace(";;", ";"), Level::Three)
        .expect("minified css")
}

pub fn create_figure(url: String, caption: Option<String>, scaling: Option<f64>) -> String {
    let (caption_html, alt_text) = match caption {
        Some(s) => (format!("<figcaption>{}</figcaption>", s), s),
        None => ("".to_string(), "No description".to_string()),
    };

    let scaling = scaling.unwrap_or(1.0);

    let dims = image_convert::get_image_dims(&url);

    let (width, height) = match dims {
        Ok(dim) => (dim.width, dim.height),
        Err(err) => {
            println!("Failed to determine dimensions for {} : {}", url, err);
            (100, 100)
        }
    };

    let height = (height as f64) * scaling;
    let width = (width as f64) * scaling;

    let height = height as usize;
    let width = width as usize;

    let new_url: String = image_convert::modify_url(url);

    let figure = format!(
        r##"<figure>
    <img src="{new_url}" width="{width}" heigth="{height}" alt="{alt_text}">
    {caption_html}
    </figure>"##
    );
    figure
}

pub fn create_index(posts: &[Post]) -> String {
    let mut index_content = "<dl>".to_string();
    for (year, year_posts) in posts
        .iter()
        .sorted_by_key(|p| p.metadata.date)
        .rev()
        .group_by(|x| x.metadata.date.year().to_string())
        .into_iter()
    {
        index_content.push_str(&format!("<dt> {} </dt> ", year));
        for post in year_posts {
            let link = format!(
                r##"<dd><a href="/posts/{}">{}</a></dd>"##,
                post.metadata.slug, post.metadata.title
            );
            index_content.push_str(&link);
        }
    }
    index_content.push_str("</dl>");
    index_content
}

pub fn parse_equation(text: &String) -> (String, bool) {
    if text.len() <= 2 {
        (text.clone(), false)
    } else if text.starts_with("$$") && text.ends_with("$$") {
        // display mode equations
        if text.len() <= 4 {
            println!("Invalid display mode equation, will skip: {}", text);
            return (text.clone(), false);
        }

        let slice = &text[2..text.len() - 2];
        let is_valid = slice.find('$').is_none();
        if !is_valid {
            panic!("Invalid display (full) mode equation: {text}")
        }
        let opts = katex::Opts::builder().display_mode(true).build().unwrap();
        let equation =
            katex::render_with_opts(slice, opts).expect("rendered display mode equation");
        (equation, true)
    } else {
        // inline equations
        let matches: Vec<_> = text.match_indices('$').collect();
        let mut indices = Vec::new();

        if matches.len() <= 1 {
            return (text.clone(), false);
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
                // loop over chunks of text and potentially multiple equations in between
                if indices.is_empty() {
                    break;
                }
                let start = indices[0];
                let end = indices[1];
                let slice: &str;

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
                    katex::render_with_opts(equation, opts).expect("rendered inline equation");
                output.push_str(&equation);
                previous_index = *end;

                indices.remove(1);
                indices.remove(0);
            }
            if previous_index + 1 < text.len() {
                output.push_str(&text[previous_index + 1..])
            }
            (output, true)
        }
    }
}

pub fn highlight_code(code: &str, language: Option<String>) -> String {
    let syntax = match language {
        Some(s) => SYNTAX_SET.find_syntax_by_token(&s),
        None => None,
    };

    let syntax = match syntax {
        Some(s) => s,
        None => match SYNTAX_SET.find_syntax_by_first_line(code) {
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
    let html = highlighted_html_for_string(
        code,
        &SYNTAX_SET,
        syntax,
        &THEME_SET.themes["base16-eighties.dark"],
    )
    .expect("parsed codeblock");

    // drop the included background color
    let start = html.find('>').expect("background color style") + 1;
    let end = html.find("</pre>").expect("background color style");
    let html = &html[start..end].trim();
    let html = format!(r##"<pre><code class="code-block">{}</code></pre>"##, html);
    html
}
