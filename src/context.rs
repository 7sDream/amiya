use {
    crate::{Middleware, Request, Response, Result},
    std::{borrow::Cow, collections::HashMap, sync::Arc},
};

/// The context middleware works on.
#[allow(missing_debug_implementations)]
pub struct Context<'x, Ex> {
    /// The incoming http request
    pub req: &'x Request,
    /// The output http response, you can directly edit it
    pub resp: &'x mut Response,
    /// User defined extra data
    pub ex: &'x mut Ex,
    pub(crate) remain_path: &'x str,
    pub(crate) router_matches: &'x mut HashMap<Cow<'static, str>, String>,
    pub(crate) tail: &'x [Arc<dyn Middleware<Ex>>],
}

impl<'x, Ex> Context<'x, Ex>
where
    Ex: Send + Sync + 'static,
{
    /// Run all inner middleware, this method drives the middleware system.
    ///
    /// Notice that you are **not must** call this func in all middleware. if you do not call it
    /// inner middleware will simply not be executed.
    ///
    /// A second call to this method on the same instance will do nothing and directly returns a
    /// `Ok(())`.
    pub async fn next(&mut self) -> Result {
        if let Some((current, tail)) = self.tail.split_first() {
            self.tail = tail;
            let next_ctx = Context {
                req: self.req,
                resp: self.resp,
                ex: self.ex,
                remain_path: self.remain_path,
                router_matches: self.router_matches,
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
    /// See [`examples/router.rs`] for a example.
    ///
    /// [`Router`]: middleware/struct.Router.html
    /// [`examples/router.rs`]: https://github.com/7sDream/amiya/blob/master/examples/router.rs
    pub fn path(&self) -> &str {
        self.remain_path
    }

    /// The path argument of `name`.
    ///
    /// Will be set if a router's any item `{name}` is matched.
    ///
    /// See *[Router - Any Item]* for more detail.
    ///
    /// ## Examples
    ///
    /// See [`examples/arg.rs`] for a example.
    ///
    /// [Router - Any Item]: middleware/struct.Router.html#any-item
    /// [`examples/arg.rs`]: https://github.com/7sDream/amiya/blob/master/examples/arg.rs
    pub fn arg<K: AsRef<str>>(&self, name: K) -> Option<&str> {
        self.router_matches.get(name.as_ref()).map(|s| s.as_str())
    }
}
