# Amiya

Yet another **experimental** middleware-based minimalism async HTTP server framework.

Working in progress, just alpha stage now, missing many features.

**API may changes every day, DO NOT use it in any condition except for test or study!**

## Hello World

Code([`examples/hello.rs`][example:hello]): 

```rust
mod common;

// m is a macro to let you easily write middleware use closure like Javascript's arrow function
// it can also convert a async fn to a middleware use the `m!(async_func_name)` syntax.
use amiya::m;

fn main() {
    // Create async runtime, start worker threads pre cpu core
    // see `examples/common/mod.rs` for code
    let ex = common::global_executor();

    // Only this stmt is Amiya related code, it sets response to some hello world texts
    let app = amiya::new().uses(m!(ctx =>
        ctx.resp.set_body(format!("Hello World from: {}", ctx.remain_path()));
    ));

    // bellow code start amiya in that runtime and blocking current thread on it
    blocking::block_on(ex.spawn(app.listen("[::]:8080"))).unwrap();
}
```

Client output: 

```bash
$ curl http://127.0.0.1:8080/we/visit/some/path -v
*   Trying 127.0.0.1:8080...
* Connected to 127.0.0.1 (127.0.0.1) port 8080 (#0)
> GET /we/visit/some/path HTTP/1.1
> Host: 127.0.0.1:8080
> User-Agent: curl/7.71.1
> Accept: */*
> 
* Mark bundle as not supporting multiuse
< HTTP/1.1 200 OK
< content-length: 36
< date: Tue, 21 Jul 2020 10:43:13 GMT
< content-type: text/plain;charset=utf-8
< 
* Connection #0 to host 127.0.0.1 left intact
Hello World from: we/visit/some/path
```

## The Middleware system

Code ([`examples/middleware.rs`][example:middleware]):

```rust
mod common;

use amiya::m;

fn main() {
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
        .uses(m!(ctx =>
            println!("new request at");
            ctx.next().await?;
            let content = ctx.resp.take_body().into_string().await.unwrap();
            println!("finish, response is: {}", content);
            ctx.resp.set_body(content);
            Ok(())
        ))
        // Let's call This middleware `Respond`
        // This middleware set tht response
        .uses(m!(ctx =>
            ctx.next().await?;
            ctx.resp.set_body("Hello World!");
            Ok(())
        ));

    blocking::block_on(ex.spawn(amiya.listen("[::]:8080"))).unwrap();
}
```

Server output:

```bash
$ cargo run --release --example taste
new request
finish, response is: Hello World!
```

Client output:

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

## Other Examples

See `examples` folder for more example with comments.

- [`examples/extra.rs`][example:extra] for how to store extra data in context
- [`examples/router.rs`][example:router] for how to use `Router` middleware
- [`examples/subapp.rs`][example:subapp] for use another `Amiya` as a middleware

## License

BSD 3-Clause Clear License, See [`LICENSE`][license].

[example:hello]: https://github.com/7sDream/amiya/blob/master/examples/hello.rs
[example:middleware]: https://github.com/7sDream/amiya/blob/master/examples/middleware.rs
[example:extra]: https://github.com/7sDream/amiya/blob/master/examples/extra.rs
[example:router]: https://github.com/7sDream/amiya/blob/master/examples/router.rs
[example:subapp]: https://github.com/7sDream/amiya/blob/master/examples/subapp.rs
[license]: https://github.com/7sDream/amiya/blob/master/LICENSE
