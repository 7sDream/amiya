use {
    crate::{middleware::Middleware, Request, Response, Result},
    std::sync::Arc,
};

#[allow(missing_debug_implementations)]
pub struct Context<'x, Ex> {
    pub req: &'x Request,
    pub resp: &'x mut Response,
    pub ex: &'x mut Ex,
    pub(crate) tail: &'x [Arc<dyn Middleware<Ex>>],
}

impl<'x, Ex> Context<'x, Ex>
where
    Ex: Send + Sync + 'static,
{
    pub async fn next(&mut self) -> Result {
        if let Some((current, tail)) = self.tail.split_first() {
            self.tail = tail;
            let next_ctx = Context { req: self.req, resp: self.resp, ex: self.ex, tail };
            current.handle(next_ctx).await
        } else {
            Ok(())
        }
    }
}
