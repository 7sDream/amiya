# Amiya

[![][doc-badge-img]][doc-gh-pages]

Amiya is a experimental middleware-based minimalism async HTTP server framework built up on the
[`smol`] async runtime.

I, a newbie to Rust's async world, start to write Amiya as a personal study project to learn 
async related concept and practice.

It's currently still working in progress and in a very early alpha stage.

API design may changes every day, **DO NOT** use it in any condition except for test or study!

## Goal

The goal of this project is try to build a (by importance order):

- Safe
- Async
- Minimalism
- Easy to use
- Easy to extend

HTTP framework for myself to write simple web services.

Amiya uses [`async-h1`] to parse and process requests, so only HTTP version 1.1 is supported for
now. HTTP 2.0 is not in goal list, at least in the near future.

Performance is **NOT** in the list too, after all, Amiya is just a experimental for now, it uses
many heap alloc (Box) and Dynamic Dispatch (Trait Object) so there may be some performance loss 
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
refer to [`examples/hello.rs`] for a minimal example of how to start [`smol`] runtime.

To run those examples, run

```bash
$ cargo run --example # show example list
$ cargo run --example hello # run hello
```

[Document of Amiya struct][doc-struct-Amiya] has some brief description of concept you need to 
understand before use it, you can check/run other examples after read it:

- Understand onion model of Amiya's middleware system: [`examples/middleware.rs`]
- How to store extra data in context: [`examples/extra.rs`]
- Use `Router` middleware for request diversion: [`examples/router.rs`]
- Use another Amiya service as a middleware: [`examples/subapp.rs`]

## License

BSD 3-Clause Clear License, See [`LICENSE`].

[doc-badge-img]: https://img.shields.io/badge/docs-on_github_pages-brightgreen?color=success&style=flat-square&logo=read-the-docs
[doc-gh-pages]: https://7sdream.github.io/amiya/master/amiya
[doc-struct-Amiya]: https://7sdream.github.io/amiya/master/amiya/struct.Amiya.html
[`smol`]: https://github.com/stjepang/smol
[`async-h1`]: https://github.com/http-rs/async-h1
[`examples/hello.rs`]: https://github.com/7sDream/amiya/blob/master/examples/hello.rs
[`examples/middleware.rs`]: https://github.com/7sDream/amiya/blob/master/examples/middleware.rs
[`examples/extra.rs`]: https://github.com/7sDream/amiya/blob/master/examples/extra.rs
[`examples/router.rs`]: https://github.com/7sDream/amiya/blob/master/examples/router.rs
[`examples/subapp.rs`]: https://github.com/7sDream/amiya/blob/master/examples/subapp.rs
[`LICENSE`]: https://github.com/7sDream/amiya/blob/master/LICENSE
