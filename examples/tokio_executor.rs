use {
    amiya::{m, Executor},
    tokio::runtime::Runtime,
};

// Due to orphan rule, we need a wrapper.
// See: https://github.com/Ixrec/rust-orphan-rules
struct TokioExecutor(Runtime);

// Implement this trait for use your custom async executor with Amiya.
impl Executor for TokioExecutor {
    fn spawn<T: Send + 'static>(
        &self, future: impl futures_lite::Future<Output = T> + Send + 'static,
    ) {
        self.0.spawn(future);
    }

    fn block_on<T>(&self, future: impl std::future::Future<Output = T>) -> T {
        self.0.block_on(future)
    }
}

fn main() {
    #[rustfmt::skip]
    let app = amiya::new()
        // With your custom executor, we can disable the "builtin-executor" feature
        // you can run this file with `--no-default-features`, try it.
        .uses(m!(ctx =>
            ctx.resp.set_body(format!("Hello World from: {}", ctx.path()));
        ));

    // Start the task in the multi-thread executor too
    app.listen("[::]:8080").unwrap();

    std::thread::park();
}
