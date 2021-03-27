// m is a macro to let you easily write middleware use closure like JavaScript's arrow function
// it can also convert a async fn to a middleware use the `m!(async_func_name)` syntax.
use amiya::m;

fn main() {
    // Only this stmt is Amiya related code, it sets response to some hello world texts
    let app = amiya::new().uses(m!(ctx =>
        ctx.resp.set_body("Hello World");
    ));

    let _ = app.listen("[::]:8080").unwrap();

    std::thread::park();
}
