# Amiya

Yet another **experimental** middleware-based minimalism async HTTP server framework, most API design learned from Tide.

Working in progress, just alpha stage now, missing many features.

## A Taste

Code:

```rust
mod common;

use amiya::{m, Amiya};

fn main() {
    // Start async worker threads pre cpu core, see `examples/common/mod.rs` for code
    common::start_smol_workers();

    // Middleware is onion model, just as NodeJs's koa framework.
    // The executed order is:
    //   - `Logger`'s code before `next()`, which print a log about request in
    //   - `Respond`'s code before `next()`, which do nothing
    //   - `Respond`'s code after `next()`, which set the response body
    //   - `Logger`'s code after `next()`, which read the response body and log it
    let amiya = Amiya::default()
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

    smol::block_on(amiya.listen("[::]:8080")).unwrap();
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

## License

UNLICENSED.
