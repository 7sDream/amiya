use {
    crate::{middleware::Middleware, Request, Response, Result},
    std::sync::Arc,
};

/// The context a middleware will works on
#[allow(missing_debug_implementations)]
pub struct Context<'x, Ex> {
    /// The incoming http request
    pub req: &'x Request,
    /// The output http response, you can directly edit it
    pub resp: &'x mut Response,
    /// User defined extra data
    pub ex: &'x mut Ex,
    pub(crate) remain_path: &'x str,
    pub(crate) tail: &'x [Arc<dyn Middleware<Ex>>],
}

impl<'x, Ex> Context<'x, Ex>
where
    Ex: Send + Sync + 'static,
{
    /// Run all inner middleware, it drives the middleware system.
    ///
    /// Notice that you are **not must** call this func in all middleware. if you do not call it
    /// inner middleware will simply not be executed.
    ///
    /// A second call to this func do nothing and return's a `Ok(())`.
    pub async fn next(&mut self) -> Result {
        if let Some((current, tail)) = self.tail.split_first() {
            self.tail = tail;
            let next_ctx = Context {
                req: self.req,
                resp: self.resp,
                ex: self.ex,
                remain_path: self.remain_path,
                tail,
            };
            current.handle(next_ctx).await
        } else {
            Ok(())
        }
    }

    /// The path the next router can match.
    ///
    /// It's differ from `Context.req.url().path()`, path returned by this method will only contains
    /// sub paths that haven't matched by any [`Router`] middleware.
    ///
    /// TODO: a example
    ///
    /// [`Router`]: middleware/struct.Router.html
    pub fn path(&self) -> &str {
        self.remain_path
    }
}
