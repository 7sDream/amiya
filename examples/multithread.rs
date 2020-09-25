mod common;

use amiya::m;

fn main() {
    #[rustfmt::skip]
    let app = amiya::new()
        // Use custom multi-thread executor
        .executor(common::GlobalMultiThreadExecutor)   
        .uses(m!(ctx =>
            ctx.resp.set_body(format!("Hello World from: {}", ctx.path()));
        ));

    // Start the task in the multi-thread executor too
    common::run(app.listen("[::]:8080")).unwrap()
}
