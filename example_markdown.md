---
title: Example
date: 2022-12-20
tags: [Tag1, Tag2]
---


Inline `code`.

Code block: 

```rust
use crate::post::Post;

#[derive(Eq, PartialEq, Debug)]
enum MultiLineType {
    CodeBlock,
    DisplayModeMath,
    None,
}
```

Block quote:
> This is a quote
> It can be long

A link: [Duck Duck Go](https://duckduckgo.com).


Inline equation $\sum^N_i x_i^2$

Display mode equation: $$ \int x^2 dx $$ or 
$$
\sum^N_i y_i^3
$$

Mix equations $x^2 + 5$ and dollars \$100 and \$5 via escaping. 

Image with caption and resized:
![wow](images/car.jpg){width=50%}


## Other things

Here's a simple footnote,[^1] and here's a longer one.[^bignote]

A table:

| Asset      | Allocation |
| ----------- | ----------- |
| Stocks      | 60       |
| Bondss and other  | 40        |
| Crypto | 0 |


### Footnotes
___


[^1]: This is the first footnote.


[^bignote]: Here's another one