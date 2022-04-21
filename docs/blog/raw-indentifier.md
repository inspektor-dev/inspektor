---
title: What is raw indentifier in rust?
description: This blog post explains about raw identifier and why it used.
slug: raw-identifier-in-rust
authors:
  - name: Poonai
    title: Maintainer of inspektor
    url: https://twitter.com/poonai_
    image_url: https://i.imgur.com/RNM7R6Q.jpg
tags: [rust]
hide_table_of_contents: false
---

Every programming language has a set of keywords that are only used for certain things. In rust, for example, the keyword `for` is used to represent looping.

Because keywords have meaning in programming languages, they cannot be used to name a function or variable. for example, the words `for` or `in` cannot be used as variable names.

Although keywords are not intended to be used to name variables, you can do so in rust by using a raw identifier.

The program below will not compile in rust because `in` is a reserved keyword.

```

#[derive(Debug)]
struct Test{
    in: String
}

fn main() {
    let a = Test{
        in: "sadf".to_string()
    };
    println!("{:?}", a);
}

```

**output:**

```
error: expected identifier, found keyword `in`
 --> src/main.rs:4:5
  |
4 |     in: String
  |     ^^ expected identifier, found keyword
  |
help: you can escape reserved keywords to use them as identifiers
  |
4 |     r#in: String
  |     ~~~~

error: expected identifier, found keyword `in`
 --> src/main.rs:9:9
  |
9 |         in: "sadf".to_string()
  |         ^^ expected identifier, found keyword
  |
help: you can escape reserved keywords to use them as identifiers
  |
9 |         r#in: "sadf".to_string()
  |         ~~~~
```
However, we can make the program work by prefixing the keyword with `r#`.

`r#` tells the compiler that the incoming token is an indentifier rather than a keyword.

```

#[derive(Debug)]
struct Test{
    r#in: String
}

fn main() {
    let a = Test{
        r#in: "sadf".to_string()
    };
    println!("{:?}", a);
}
```

**output:**

```
Test { in: "sadf" }
```

It's very useful for rust because it allows rust to introduce new keywords.

Assume we have a crate built with the 2015 rust edition that exposes the identifier `try`. Later, `try` was reserved for a feature in the 2018 edition. As a result, we must use a raw identifier to call `try`

**reference**
- https://doc.rust-lang.org/rust-by-example/compatibility/raw_identifiers.html