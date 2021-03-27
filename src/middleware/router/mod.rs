use {
    crate::{Context, Middleware, Result, StatusCode},
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
        /// Enter endpoint edit environment.
        #[must_use]
        pub fn endpoint(
            self,
        ) -> $crate::middleware::router::setter::RouterSetter<
            Self,
            $crate::middleware::router::set_which::SetEndpoint,
            $ex,
        > {
            $crate::middleware::router::setter::RouterSetter::new_endpoint_setter(self)
        }

        /// Add a new item for `path` to router table and enter endpoint edit environment of
        /// that item.
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

        /// Enter fallback edit environment.
        #[must_use]
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

/// The middleware for request diversion by path.
///
/// ## Concepts
///
/// There also are some important concepts need to be described first.
///
/// ### [`Router`]
///
/// A [`Router`] is some component that dispatch your [`Request`] to different handler(inner
/// middleware) by looking it's [`path`].
///
/// So a router may store several middleware and choose zero or one of them for you when a request
/// comes. If [`Request`]'s path match a router table item, it delegate [`Context`] to that
/// corresponding middleware and do nothing else. If no item matches, it set [`Response`] to `404
/// Not Found` and no stored middleware will be executed.
///
/// ### Router Table
///
/// [`Router`] has a `Path => Middleware` table to decided which middleware is respond for a
/// request.
///
/// Each table item has different path, if you set a path twice, the new one will replace the
/// first.
///
/// Each path, is a full part in path when split by `/`, that is, if you set a item `"abc" =>
/// middleware A`, the path `/abcde/somesub` will not be treated as a match. Only `/abc`, `/abc/`,
/// `/abc/xxxxx/yyyy/zzzz` will.
///
/// You can use [`at`] method to edit router table, for example: `at("abc")` will start a router
/// table item edit environment for sub path `/abc`.
///
/// ### Any Item
///
/// There is a special router item called `any`, you can set it by use `at("{arg_name}")`.
///
/// This item will match when all router table items do not match the remain path, and remain path
/// is not just `/`. That is, remain path have sub folder.
///
/// At this condition, the `any` item will match next sub folder, and store this folder name as a
/// match result in  [`Context`], you can use [`Context::arg`] to get it.
///
/// see [`examples/arg.rs`] for a example code.
///
/// ### Endpoint
///
/// Except router table, [`Router`] has a endpoint middleware to handler condition that no more remain
/// path can be used to determine which table item should be used.
///
/// A example:
///
/// ```
/// # use amiya::{middleware::Router, m};
/// let router = Router::new()
///     .endpoint()
///     .get(m!(ctx => ctx.resp.set_body("hit endpoint");))
///     .at("v1").is(m!(ctx => ctx.resp.set_body("hit v1");));
/// let main_router = Router::new().at("api").is(router);
///
/// let amiya = amiya::new().uses(main_router);
/// ```
///
/// The `hit endpoint` will only be returned when request path is exactly `/api`, because after
/// first match by `main_router`, remain path is empty, we can't match sub path on empty string.
///
/// ### Fallback
///
/// With the example above, if request path is `/api/`, the endpoint is not called, and because
/// the `v1` item do not match remain path `/` too, so there is a mismatch.
///
/// If we do not add any code, this request will get a `404 Not Found` response. We have two option
/// too add a handler for this request:
///
/// 1. use a empty path router table item: `.at("").uses(xxxx)`.
/// 2. use a fallback handler: `.fallback().uses(xxxxx)`.
///
/// When remain path is not empty, but we can't find a matched router item, the fallback handler
/// will be executed(only if we have set one, of course).
///
/// So if we choose the second option, the fallback is respond to all mismatched item, sometime
/// this is what you want, and sometime not. Make sure choose the approach meets your need.
///
/// ## API Design
///
/// Because router can be nest, with many many levels, we need many code, many temp vars to build
/// a multi level router. For reduce some ugly code, we designed a fluent api to construct this
/// tree structure.
///
/// As described above, a [`Router`] has three property:
///
/// - Endpoint handler
/// - Router table
/// - Fallback handler
///
/// And we have three methods foo them:
///
/// - [`endpoint`]
/// - [`at`]
/// - [`fallback`]
///
/// ### Editing Environment
///
/// Let's start at the simplest method [`fallback`].
///
/// When we call [`fallback`] on a router, we do not set the middleware for this property, instead,
/// we enter the fallback editing environment of this router.
///
/// In a edit environment, we can use several method to finish this editing and exit environment.
///
/// - any method of a [`MethodRouter`] like [`get`], [`post`], [`delete`], etc..
/// - `uses` method of that non-public environment type.
///
/// A finish method consumes the environment, set the property of editing target and returns it. So
/// we can enter other properties' editing environment to make more changes to it.
///
/// The [`endpoint`] editing environment is almost the same, except it sets the endpoint handler.
///
/// But [`at`] method has a little difference. It does not enter router table editing environment of
/// `self`, but enter the [`endpoint`] editing environment of that corresponding router table item's
/// middleware, a [`Router`] by default.
///
/// If we finish this editing, it returns a type representing that Router with `endpoint` set by the
/// finish method. But we do not have to finish it. We can use `is` method to use a
/// custom middleware in that router table item directly.
///
/// And a Router table item's endpoint editing environment also provided `fallback` and `at` method
/// to enter the default sub Router's editing environment quickly without endpoint be set.
///
/// if we finish set this sub router, a call of `done` method can actually add this item to parent's
/// router table and returns parent router.
///
/// example:
///
/// ```
/// # use amiya::{Context, middleware::Router, Result, m};
///
/// async fn xxx(ctx: Context<'_, ()>) -> Result { Ok(()) } // some middleware func
///
/// #[rustfmt::skip]
/// let router = Router::new()
///     // | this enter router table item "root"'s default router's endpoint
///     // v editing environment
///     .at("root")
///         // | set "root" router's endpoint only support GET method, use middleware xxx
///         // v this will return a type representing sub router
///         .get(m!(xxx))
///         // | end
///         .fallback()  // <-- enter sub router's fallback editing endpoint
///         // | set sub router's endpoint to use middleware xxx directly
///         // v This method returns the type representing sub router again
///         .uses(m!(xxx))
///         // | enter sub sub router's endpoint editing environment
///         // v
///         .at("sub")
///             .is(m!(xxx))    // `is` make "sub" path directly uses xxx
///         .done()             // `done` finish "root" router editing
///     .at("another")          // we can continue add more item to the Router
///         .is(m!(xxx));       // but for short we use a is here and finish router build.
/// ```
///
/// Every [`at`] has a matched `done` or `is`, remember this, then you can use this API to build a
/// router tree without any temp variable.
///
/// Because that `rustfmt` align code using `.`, so all chain method call will have same indent by
/// default. No indent means no multi level view, no level means we need to be very careful when add
/// new path to old router. So I recommend use [`#[rustfmt::skip]`][rustfmt::skip] to prevent
/// `rustfmt` to format the router creating code section and indent router code by hand.
///
/// ## Examples
///
/// see [`examples/router.rs`], [`examples/arg.rs`] and [`examples/subapp.rs`].
///
/// [`Router`]: #main
/// [`Request`]: ../struct.Request.html
/// [`path`]: ../struct.Context.html#method.path
/// [`Context`]: ../struct.Context.html
/// [`Context::arg`]: ../struct.Context.html#method.arg
/// [`Response`]: ../struct.Response.html
/// [`endpoint`]: #method.endpoint
/// [`at`]: #method.at
/// [`fallback`]: #method.fallback
/// [`MethodRouter`]: struct.MethodRouter.html
/// [`get`]: struct.MethodRouter.html#method.get
/// [`post`]: struct.MethodRouter.html#method.post
/// [`delete`]: struct.MethodRouter.html#method.delete
/// [rustfmt::skip]: https://github.com/rust-lang/rustfmt#tips
/// [`examples/arg.rs`]: https://github.com/7sDream/amiya/blob/master/examples/arg.rs
/// [`examples/router.rs`]: https://github.com/7sDream/amiya/blob/master/examples/router.rs
/// [`examples/subapp.rs`]: https://github.com/7sDream/amiya/blob/master/examples/subapp.rs
#[allow(missing_debug_implementations)]
pub struct Router<Ex> {
    endpoint: Option<Box<dyn Middleware<Ex>>>,
    fallback: Option<Box<dyn Middleware<Ex>>>,
    any: Option<(Cow<'static, str>, Box<dyn Middleware<Ex>>)>,
    table: HashMap<Cow<'static, str>, Box<dyn Middleware<Ex>>>,
}

impl<Ex> Default for Router<Ex> {
    fn default() -> Self {
        Self { endpoint: None, fallback: None, any: None, table: HashMap::new() }
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
        let path = path.into();
        if path.starts_with('{') && path.ends_with('}') {
            match path {
                Cow::Owned(path) => {
                    let key = &path[1..path.len() - 1];
                    self.any.replace((Cow::from(key.to_string()), Box::new(middleware)));
                }
                Cow::Borrowed(path) => {
                    let key = &path[1..path.len() - 1];
                    self.any.replace((Cow::from(key), Box::new(middleware)));
                }
            }
        } else {
            self.table.insert(path, Box::new(middleware));
        }
    }
}

impl<Ex> Router<Ex> {
    /// Create new Router middleware.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    impl_router_like_pub_fn! { Ex }
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
            let path = &ctx.remain_path[1..];
            for (target_path, sub_router) in &self.table {
                if path.starts_with(target_path.as_ref()) {
                    if path.len() == target_path.len() {
                        ctx.remain_path = "";
                    } else if path[target_path.len()..].starts_with('/') {
                        ctx.remain_path = &path[target_path.len()..];
                    } else {
                        continue;
                    }
                    return sub_router.handle(ctx).await;
                }
            }

            if let Some((ref k, ref any)) = self.any {
                if !path.is_empty() && !path.starts_with('/') {
                    let next_slash = path.find('/');
                    #[allow(clippy::option_if_let_else)] // use same ctx as mutable
                    let pos = if let Some(pos) = next_slash {
                        ctx.remain_path = &path[pos..];
                        pos
                    } else {
                        ctx.remain_path = "";
                        path.len()
                    };
                    let value = &path[0..pos];
                    ctx.router_matches.insert(k.clone(), value.to_string());
                    return any.handle(ctx).await;
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
