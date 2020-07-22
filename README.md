# Amiya

Amiya is a experimental middleware-based minimalism async HTTP server framework built up on the
[`smol`] async runtime.

As a newbie to rust's async world, It's a personal study project to learn async related concept
and practice.

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

Amiya uses [`async-h1`] to parse and process the request, so only HTTP version 1.1 is supported
for now. HTTP 2.0 is not in goal list, at least for the near future.

Performance is NOT in the list too, after all, it's just a experimental. Amiya use many heap alloc
(Box) and Dynamic Dispatch (Trait Object) so there maybe are some performance reduce compare to use
`async-h1` directly.

## Examples

To start a very simple HTTP service to return Hello World to the client in all path:

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

Notice any future need a async runtime to do this, and that's not amiya's goal too. But you
can refer to [`examples/hello.rs`] for a minimal example of how to start [`smol`] runtime.

To run those examples, run

```bash
$ cargo run --example # show example list
$ cargo run --example hello # run hello
```

You can check/run other examples for:

- Understand onion model of Amiya's middleware system: [`examples/middleware.rs`]
- How to store extra data in context: [`examples/extra.rs`]
- Use `Router` middleware for request diversion: [`examples/router.rs`]
- Use another Amiya service as a middleware: [`examples/subapp.rs`]

## License

BSD 3-Clause Clear License, See [`LICENSE`].

[`smol`]: https://github.com/stjepang/smol
[`async-h1`]: https://github.com/http-rs/async-h1
[`examples/hello.rs`]: https://github.com/7sDream/amiya/blob/master/examples/hello.rs
[`examples/middleware.rs`]: https://github.com/7sDream/amiya/blob/master/examples/middleware.rs
[`examples/extra.rs`]: https://github.com/7sDream/amiya/blob/master/examples/extra.rs
[`examples/router.rs`]: https://github.com/7sDream/amiya/blob/master/examples/router.rs
[`examples/subapp.rs`]: https://github.com/7sDream/amiya/blob/master/examples/subapp.rs
[`LICENSE`]: https://github.com/7sDream/amiya/blob/master/LICENSE
