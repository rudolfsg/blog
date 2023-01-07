---
title: Static site generator in Rust
date: 2022-12-27
tags: [Programming, Rust]
---

To become familiar with Rust I implemented a static site generator as my first project. Below is a set of notes which I would've found helpful when I started. Code is available [here](https://github.com/rudolfsg/blog).

## Static sites

I've always liked the simplicity of static websites where all pages are generated ahead of time instead of waiting on a back-end server to generate pages on demand. Static sites are also a perfect fit for content delivery networks (CDN) which enables great performance. 

Static site generators usually accept a simple text format like markdown as input to generate pages. A simple input format eliminates most of the tool/framework lock-in and substantially increases the probability of your content being usable 10 or 20 years down the line. This is probably why note-taking software like [Obsidian](https://obsidian.md/) have become so popular. 

## The plan

Write a static site generator from scratch in Rust for a blog that I might use $n \geq 1$ times. It should
1. be simple to modify
2. produce a fast, minimal site
3. support code syntax highlighting and $\LaTeX$ equations


# Your own site generator

To generate a static site we:
1. Parse markdown files and transform into html with a library like [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)
2. Insert them into html templates with a templating library; [tera](https://github.com/Keats/tera) is a popular choice
3. Create site index, copy over assets

To make the site pretty we add a `style.css`, a font or two and we're done with the basics. 

## Syntax highlighting

By default `pulldown-cmark` detects code blocks, however, it won't highlight the syntax. Luckily this is simple to adjust: `pulldown-cmark` outputs _events_ during parsing, one of which is a [code block](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/enum.Tag.html#variant.CodeBlock). Once we hit this event we can override the default behaviour by feeding the code string block into a [highlighter](https://crates.io/crates/syntect) and outputting colorised html. We can also set the code block font to [Jetbrains Mono](https://www.jetbrains.com/lp/mono/) which looks better and supports ligatures.

This gives us nice code snippets:

```rust
pub struct EventIterator<'a, I: Iterator<Item = Event<'a>>> {
    parser: MultiPeek<I>,
    has_katex: bool,
    image_scale: HashMap<String, f64>,
}
```

## Images 

`pulldown-cmark` parses embedded images just fine, but it's worthwhile to add some extra logic to make the embedding process smoother and improve site performance:

* By default the image caption won't be transferred to html even though the caption is captured as a part of the `Image` event. Like with syntax highlighting we can capture the parsing event and output adjusted html which includes the caption 
* When writing a post I want to paste images from the clipboard and not worry about resizing them manually. So I adjusted the image parser to look for an extra tag after the image: For example, `![cool](images/backflip.jpg){width=50%}` would resize the image by half which makes life easier and saves a lot of bandwidth
* To further reduce bandwidth I've added conversion of `png` and `jpeg` to `webp`

Now the above command works:

![cool](images/backflip.jpg){width=70%}

## Math equations

To render math equations we can use [katex](https://katex.org/). But unlike code blocks `pulldown-cmark` won't automatically detect or render math due to lack of standardisation [^math], so we have to roll our own.

To delineate math blocks in markdown I use single dollar signs for inline math `$ x^2 $` and double for _display mode_ math `$$ x^2 $$`. We can then detect math blocks by checking each paragraph for `$`.

To render the equations we could leave the math blocks alone and include `katex` auto-render javascript in our site. But in the spirit of static generation, we can render the equations ahead of time with [katex rust bindings](https://docs.rs/katex/latest/katex/) and include the necessary fonts/css in our site instead of going to their CDN. 

And now we can do this:

$$ \int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi} $$


## Other things

Self-hosting your assets like fonts and icons [may improve performance](https://www.tunetheweb.com/blog/should-you-self-host-google-fonts/) since it avoids going to a third-party CDN. Besides, having a fully self-contained site is nice.

Modern font formats like WOFF2 take less space and nowadays have [good support](https://caniuse.com/woff2) making them an easy choice. Font size can be further improved via subsetting. [^font] Using a variable font keeps the codebase cleaner as you don't have to store separate files for italic, bold, regular etc. [Fontshare](https://www.fontshare.com/) is a good resource. 
 


### Footnotes
___

[^math]: While Markdown is not a proper specification, CommonMark is but the [spec](https://spec.commonmark.org/) doesn't mention math once. Unfortunately, the popular `$` delimiter allow for ambiguities which is a no-no for a spec.

For `pulldown-cmark` there's lots of [discussion on the issue](https://github.com/raphlinus/pulldown-cmark/issues/6) going back to 2015 (!). However, there's good progress on [an extension](https://github.com/raphlinus/pulldown-cmark/pull/622). 

[^font]: This [article on subsetting](https://markoskon.com/creating-font-subsets/) is great. However, there don't seem to be great rust libraries to do it seamlessly and it might break things if you're not careful, all to save what is likely less than ~50kb.
