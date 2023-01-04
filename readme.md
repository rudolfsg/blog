# Blog

Simple, minimal static site generator for a blog written in Rust.

Features:
* (almost) no javascript
* code syntax highlighting with `syntect`
* server-side rendered math equations with `katex`
* RSS feed generation
* Embedded jpg and png compressed with webp
  * Optional automatic resizing by placing a `{width=50%}` tag after the embed link
* Markdown parsing and image resizing parallelised with `rayon`

## Usage

1. Edit markdown files in `/posts`
2. Run `cargo run --release`
3. Check results by running a webserver.
 Don't forget to serve files from root: 
  `cd build`
  `python3 -m http.server 1234`
  