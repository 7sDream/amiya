# Amiya

[![][github-badge-img]][github-home] [![][doc-badge-img]][doc-home]
[![][workflow-badge-image]][workflow-page]

Amiya is a experimental middleware-based minimalism async HTTP server framework built up on the
[`smol`] async runtime.

I, a newbie to Rust's async world, start to write Amiya as a personal study project to learn
async related concept and practice.

It's currently still working in progress and in a very early alpha stage.

API design may changes every day, **DO NOT** use it in any condition except for test or study!

## Goal

The goal of this project is try to build a (by importance order):

- Safe, with `#![forbid(unsafe_code)]`
- Async
- Minimalism
- Easy to use
- Easy to extend

HTTP framework for myself to write simple web services.

Amiya uses [`async-h1`] to parse and process requests, so only HTTP version 1.1 is supported for
now. HTTP 1.0 or 2.0 is not in goal list, at least in the near future.

Performance is **NOT** in the list too, after all, Amiya is just a experimental for now, it uses
many heap alloc (Box) and dynamic dispatch (Trait Object) so there may be some performance loss
compare to use `async-h1` directly.

## Examples

To start a very simple HTTP service that returns `Hello World` to the client in all paths:

```rust
use amiya::m;

fn main() {
    let app = amiya::new().uses(m!(ctx =>
        ctx.resp.set_body(format!("Hello World from: {}", ctx.path()));
    ));

    let fut = app.listen("[::]:8080");

    // ... start a async runtime and block on `fut` ...
}
```

You can await or block on this `fut` to start the service.

Notice any future need a async runtime to run, and that's not amiya's goal too. But you can
refer to [`examples/hello.rs`] for a minimal example of how to use [`smol`] runtime.

To run those examples, run

```bash
$ cargo run --example # show example list
$ cargo run --example hello # run hello
```

Top level document of crate has [a brief description of concepts][doc-concepts] we used in this
framework, I recommend read it first, and then check those examples to get a more intuitive
understanding:

- Understand onion model of Amiya's middleware system: [`examples/middleware.rs`]
- Use a custom type as middleware: [`examples/measurer.rs`]
- Store extra data in context: [`examples/extra.rs`]
- Use `Router` middleware for request diversion: [`examples/router.rs`]
- Parse query string to json value or custom struct: [`examples/query.rs`]
- Parse body(www-form-urlencoded) to json value or custom struct: [`examples/urlencoded.rs`]
- Match part of path as an argument: [`examples/arg.rs`]
- Use another Amiya app as middleware: [`examples/subapp.rs`]

## License

BSD 3-Clause Clear License, See [`LICENSE`].

[github-badge-img]: https://img.shields.io/badge/Github-7sDream%2Famiya-8da0cb?style=for-the-badge&labelColor=555555&logo=github
[github-home]: https://github.com/7sDream/amiya
[doc-badge-img]: https://img.shields.io/badge/docs-on_github_pages-66c2a5?style=for-the-badge&labelColor=555555&logo=read-the-docs
[doc-home]: https://7sdream.github.io/amiya/master/amiya
[workflow-badge-image]: https://img.shields.io/github/workflow/status/7sDream/amiya/DocPublishToGithubPages/master?style=for-the-badge&logo=github-actions
[workflow-page]: https://github.com/7sDream/amiya/actions?query=workflow%3ADocPublishToGithubPages
[doc-concepts]: https://7sdream.github.io/amiya/master/amiya#concepts
[`smol`]: https://github.com/stjepang/smol
[`async-h1`]: https://github.com/http-rs/async-h1
[`examples/hello.rs`]: https://github.com/7sDream/amiya/blob/master/examples/hello.rs
[`examples/middleware.rs`]: https://github.com/7sDream/amiya/blob/master/examples/middleware.rs
[`examples/measurer.rs`]: https://github.com/7sDream/amiya/blob/master/examples/measurer.rs
[`examples/extra.rs`]: https://github.com/7sDream/amiya/blob/master/examples/extra.rs
[`examples/query.rs`]: https://github.com/7sDream/amiya/blob/master/examples/query.rs
[`examples/urlencoded.rs`]: https://github.com/7sDream/amiya/blob/master/examples/urlencoded.rs
[`examples/router.rs`]: https://github.com/7sDream/amiya/blob/master/examples/router.rs
[`examples/arg.rs`]: https://github.com/7sDream/amiya/blob/master/examples/arg.rs
[`examples/subapp.rs`]: https://github.com/7sDream/amiya/blob/master/examples/subapp.rs
[`LICENSE`]: https://github.com/7sDream/amiya/blob/master/LICENSE
