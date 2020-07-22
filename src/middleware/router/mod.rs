use {
    super::Middleware,
    crate::{Context, Result, StatusCode},
    async_trait::async_trait,
    std::{borrow::Cow, collections::HashMap},
};

mod method;
mod set_which;
mod setter;

pub use method::MethodRouter;

pub trait RouterLike<Ex>: Sized {
    fn set_endpoint<M: Middleware<Ex> + 'static>(&mut self, middleware: M);
    fn set_fallback<M: Middleware<Ex> + 'static>(&mut self, middleware: M);
    fn insert_to_router_table<P: Into<Cow<'static, str>>, M: Middleware<Ex> + 'static>(
        &mut self, path: P, middleware: M,
    );
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_router_like_pub_fn {
    ($ex: ty) => {
        /// Enter endpoint edit environment
        pub fn endpoint(
            self,
        ) -> $crate::middleware::router::setter::RouterSetter<
            Self,
            $crate::middleware::router::set_which::SetEndpoint,
            $ex,
        > {
            $crate::middleware::router::setter::RouterSetter::new_endpoint_setter(self)
        }

        /// Enter **endpoint** edit environment for router table's `path` item
        pub fn at<P: Into<Cow<'static, str>>>(
            self, path: P,
        ) -> $crate::middleware::router::setter::RouterSetter<
            $crate::middleware::router::setter::RouterSetter<
                Self,
                $crate::middleware::router::set_which::SetTableItem,
                $ex,
            >,
            $crate::middleware::router::set_which::SetEndpoint,
            $ex,
        > {
            $crate::middleware::router::setter::RouterSetter::new_router_table_setter(self, path)
                .endpoint()
        }

        /// Enter fallback edit environment
        pub fn fallback(
            self,
        ) -> $crate::middleware::router::setter::RouterSetter<
            Self,
            $crate::middleware::router::set_which::SetFallback,
            $ex,
        > {
            $crate::middleware::router::setter::RouterSetter::new_fallback_setter(self)
        }
    };
}

/// Router middleware for request diversion
#[allow(missing_debug_implementations)]
pub struct Router<Ex> {
    endpoint: Option<Box<dyn Middleware<Ex>>>,
    fallback: Option<Box<dyn Middleware<Ex>>>,
    table: HashMap<Cow<'static, str>, Box<dyn Middleware<Ex>>>,
}

impl<Ex> Default for Router<Ex> {
    fn default() -> Self {
        Self { endpoint: None, fallback: None, table: HashMap::new() }
    }
}

impl<Ex> RouterLike<Ex> for Router<Ex> {
    fn set_endpoint<M: Middleware<Ex> + 'static>(&mut self, middleware: M) {
        self.endpoint.replace(Box::new(middleware));
    }

    fn set_fallback<M: Middleware<Ex> + 'static>(&mut self, middleware: M) {
        self.fallback.replace(Box::new(middleware));
    }

    fn insert_to_router_table<P: Into<Cow<'static, str>>, M: Middleware<Ex> + 'static>(
        &mut self, path: P, middleware: M,
    ) {
        self.table.insert(path.into(), Box::new(middleware));
    }
}

impl<Ex> Router<Ex> {
    /// Create new Router middleware
    pub fn new() -> Self {
        Self::default()
    }

    impl_router_like_pub_fn! { Ex }
}

/// Create new Router middleware, see [`Router`] for detail.
///
/// [`Router`]: struct.Router.html
pub fn router<Ex>() -> Router<Ex> {
    Router::default()
}

#[async_trait]
impl<Ex> Middleware<Ex> for Router<Ex>
where
    Ex: Send + Sync + 'static,
{
    async fn handle(&self, mut ctx: Context<'_, Ex>) -> Result<()> {
        if ctx.remain_path.is_empty() {
            if let Some(ref endpoint) = self.endpoint {
                return endpoint.handle(ctx).await;
            }
        } else {
            for (target_path, sub_router) in &self.table {
                if ctx.remain_path.starts_with(target_path.as_ref()) {
                    if ctx.remain_path.len() == target_path.len() {
                        ctx.remain_path = "";
                    } else if ctx.remain_path[target_path.len()..].starts_with('/') {
                        ctx.remain_path = &ctx.remain_path[(target_path.len() + 1)..];
                    } else {
                        continue;
                    }
                    return sub_router.handle(ctx).await;
                }
            }

            if let Some(ref fallback) = self.fallback {
                return fallback.handle(ctx).await;
            }
        }

        ctx.resp.set_status(StatusCode::NotFound);
        Ok(())
    }
}
