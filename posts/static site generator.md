---
title: Static site generator in Rust
date: 2022-12-27
tags: [Programming, Rust]
---


Deciding on a starter project to learn a new language can be tricky. Initially I tried [advent of code](https://adventofcode.com/) but soon found myself not really applying the ideas from [the book](https://doc.rust-lang.org/book/). Then I stumbled upon a discussion on static site generators and so decided to [give it a go](https://github.com/rudolfsg/blog).

## Static sites

I've always liked the simplicity of static websites where all pages are generated ahead of time instead of waiting on a back-end server to generate pages on demand. Static content is also a perfect fit for content delivery networks (CDN) which gives good performance. 

Static site generators usually accept a simple text format like markdown as input to generate pages. A simple input format eliminates most of the tool/framework lock-in and substantially increases the probability of your content being usable 10 or 20 years down the line. This is probably why note taking software like [Obsidian](https://obsidian.md/) have become so popular. 

## The plan

Write a static site generator from scratch for a blog that I might use $n \geq 1$ times. It should be
1. simple to modify
2. good performance
3. visually minimalistic but (hopefully) not boring
4. support code syntax highlighting, $\LaTeX$ equations

# Your own site generator

Steps to generate a site:
1. Parse markdown files and transform into html with a library like [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)
2. Insert them into an html template with a templating library, for Rust [tera](https://github.com/Keats/tera) is a popular choice
3. Create site index, copy over assets

To make the site pretty we add a `style.css` and a font or two and we're done with the basics. 

## Syntax highlighting

By default `pulldown-cmark` detects code blocks, however, it won't highlight the syntax. Luckily this is simple to adjust - once we've detected a code block we override the default behaviour by feeding the code block into a [highlighter](https://crates.io/crates/syntect) and outputting colorised html. Finally, I've set code block font to [Jetbrains Mono](https://www.jetbrains.com/lp/mono/) which supports ligatures.

Now we can have nice code snippets:

```rust
pub struct EventIterator<'a, I: Iterator<Item = Event<'a>>> {
    parser: MultiPeek<I>,
    has_katex: bool,
    image_scale: HashMap<String, f64>,
}
```

## Images 

`pulldown-cmark` parses embedded images just fine, but it might be worth to add some extra logic:

* by default the image caption from markdown won't be transferred to html even though the caption is captured as a part of the `Image` event. Like with syntax highlighting we can adjust the html to include the caption 
* when writing a post I might paste an image from clipboard which might be too big. Manual resizing is no fun, so I adjusted the image parser to look for an extra tag after the image `[cool] (images/back flip. jpg) {width=50%}` which would resize the image by half and preserve the aspect ratio 
* I might embed images taken from my phone or from the Internet which have large file size. To reduce bandwidth I've added compression to webp - convert png losslessly, jpg with high quality. Adding gif to webm/mp4 would be a good future improvement 

These adjustments make the embedding process smoother and improve site performance. 

## Math equations

To render math equations we can use [katex](https://katex.org/). But unlike code blocks `pulldown-cmark` won't automatically detect or render math due to lack of standardisation [^math], so we have to roll our own.

To delineate math blocks in markdown I use single dollar signs for inline math `$ x^2 $` and double for _display mode_ math `$$ x^2 $$`. We can then detect math blocks by checking each paragraph for `$`.

To render the equations we could leave the math blocks alone and include `katex` auto-render javascript in our site. But in the spirit of static generation, we can render the equations ahead of time with [katex rust bindings](https://docs.rs/katex/latest/katex/) and include the necessary fonts/css in our site instead of going to their CDN. 

Now we can do things like

$$ \int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi} $$


## Making things fast(er)(?)

First of all, a [good idea](https://www.tunetheweb.com/blog/should-you-self-host-google-fonts/) might be to self-host your fonts in a modern format like WOFF2 to avoid going to a third-party CDN. Pure minimalistics might enjoy subsetting [^font], otherwise checking out [fontshare](https://www.fontshare.com/) and using a variable font is a good start.



### Footnotes
___

[^math]: While Markdown is not a proper specification, CommonMark is but the [spec](https://spec.commonmark.org/) doesn't mention math once. Unfortunately, the popular `$` delimiter allow for ambiguities which is a no-no for a spec.

For `pulldown-cmark` there's lots of [discussion on the issue](https://github.com/raphlinus/pulldown-cmark/issues/6) going back to 2015 (!). However, there's good progress on [an extension](https://github.com/raphlinus/pulldown-cmark/pull/622) which is still ongoing at time of writing. 

[^font]: [This](https://markoskon.com/creating-font-subsets/) is a great article on subsetting. However, there don't seem to be great rust libraries for this, it takes some effort and might break things if you're not careful, all to save less than ~50kb.
