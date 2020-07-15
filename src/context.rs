use {
    crate::{middleware::Middleware, Request, Response, Result, StatusCode},
    std::sync::Arc,
};

#[allow(missing_debug_implementations)]
pub struct Context<'x> {
    pub req: Request,
    pub(crate) tail: &'x [Arc<dyn Middleware>],
}

impl<'x> Context<'x> {
    pub async fn next(mut self) -> Result {
        if let Some((current, tail)) = self.tail.split_first() {
            self.tail = tail;
            current.handle(self).await
        } else {
            Ok(Response::new(StatusCode::Ok))
        }
    }
}
