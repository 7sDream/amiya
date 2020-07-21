# Amiya

Yet another **experimental** middleware-based minimalism async HTTP server framework.

Working in progress, just alpha stage now, missing many features.

**API may changes every day, DO NOT use it in any condition except for test or study!**

## A Taste

Code:

```rust
mod common;

// m is a macro to let you easily write middleware use closure like Javascript's arrow function
// it can also convert a async fn to a middleware use the `m!(async_func_name)` syntax.
use amiya::m;

fn main() {
    // Start async worker threads pre cpu core, see `examples/common/mod.rs` for code
    let ex = common::global_executor();

    // The middleware system of Amiya uses onion model, just as NodeJs's koa framework.
    // The executed order is:
    //   - `Logger`'s code before `next()`, which print a log about request in
    //   - `Respond`'s code before `next()`, which do nothing
    //   - `Respond`'s code after `next()`, which set the response body
    //   - `Logger`'s code after `next()`, which read the response body and log it
    let amiya = amiya::new()
        // Let's call This middleware `Logger`
        // `ctx.next().await` will return after all inner middleware be executed
        // so the `content` will be "Hello World" , which is set by next middleware.
        .uses(m!(ctx => {
            println!("new request at");
            ctx.next().await?;
            let content = ctx.resp.take_body().into_string().await.unwrap();
            println!("finish, response is: {}", content);
            ctx.resp.set_body(content);
            Ok(())
        }))
        // Let's call This middleware `Respond`
        // This middleware set tht response
        .uses(m!(ctx => {
            ctx.next().await?;
            ctx.resp.set_body("Hello World!");
            Ok(())
        }));

    blocking::block_on(ex.spawn(amiya.listen("[::]:8080"))).unwrap();
}
```

Log:

```bash
$ cargo run --release --example taste
new request
finish, response is: Hello World!
```

Curl:

```bash
$ curl 'http://127.0.0.1:8080/' -v
*   Trying 127.0.0.1:8080...
* Connected to 127.0.0.1 (127.0.0.1) port 8080 (#0)
> GET / HTTP/1.1
> Host: 127.0.0.1:8080
> User-Agent: curl/7.71.1
> Accept: */*
>
* Mark bundle as not supporting multiuse
< HTTP/1.1 200 OK
< content-length: 12
< date: Thu, 16 Jul 2020 09:50:58 GMT
< content-type: text/plain;charset=utf-8
<
* Connection #0 to host 127.0.0.1 left intact
Hello World!
```

## Examples

See `examples` folder for more example with comments.

- [`examples/extra.rs`][example:extra] for how to store extra data in context
- [`examples/router.rs`][example:router] for how to Use `Router` middleware
- [`examples/subapp.rs`][example:subapp] for use another `Amiya` as a middleware

## License

BSD 3-Clause Clear License, See [`LICENSE`][license].

[example:extra]: https://github.com/7sDream/amiya/blob/master/examples/extra.rs
[example:router]: https://github.com/7sDream/amiya/blob/master/examples/router.rs
[example:subapp]: https://github.com/7sDream/amiya/blob/master/examples/subapp.rs
[license]: https://github.com/7sDream/amiya/blob/master/LICENSE
