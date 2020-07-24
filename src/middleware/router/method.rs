use {
    crate::{Context, Method, Middleware, Result, StatusCode},
    async_trait::async_trait,
    std::{
        collections::HashMap,
        fmt::{self, Debug, Formatter},
        sync::Arc,
    },
};

static ALL_METHODS: &'static [Method] = &[
    Method::Get,
    Method::Head,
    Method::Post,
    Method::Put,
    Method::Delete,
    Method::Connect,
    Method::Options,
    Method::Trace,
    Method::Patch,
];

#[doc(hidden)]
#[macro_export]
macro_rules! impl_method {
    ($(#[$outer:meta])*
    $func_name: ident : $method: expr => $ret: ty) => {
        $(#[$outer])*
        pub fn $func_name<M: Middleware<Ex> + 'static>(self, middleware: M) -> $ret {
            self.method($method, middleware)
        }
    };

    ($($(#[$outer:meta])*
    $func_name: ident : $method: expr),* $(,)? => $ret: ty) => {
        $(impl_method!{$(#[$outer])* $func_name: $method => $ret})+
    };
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! impl_all_http_method {
    ($ret: ty) => {
        impl_method! {
            /// A shortcut of `self.method(Method::Get, middleware)`, see [`Self::method`].
            ///
            /// [`Self::method`]: #method.method
            get: Method::Get,
            /// A shortcut of `self.method(Method::Head, middleware)`, see [`Self::method`].
            ///
            /// [`Self::method`]: #method.method
            head: Method::Head,
            /// A shortcut of `self.method(Method::Post, middleware)`, see [`Self::method`].
            ///
            /// [`Self::method`]: #method.method
            post: Method::Post,
            /// A shortcut of `self.method(Method::Put, middleware)`, see [`Self::method`].
            ///
            /// [`Self::method`]: #method.method
            put: Method::Put,
            /// A shortcut of `self.method(Method::Delete, middleware)`, see [`Self::method`].
            ///
            /// [`Self::method`]: #method.method
            delete: Method::Delete,
            /// A shortcut of `self.method(Method::Connect, middleware)`, see [`Self::method`].
            ///
            /// [`Self::method`]: #method.method
            connect: Method::Connect,
            /// A shortcut of `self.method(Method::Options, middleware)`, see [`Self::method`].
            ///
            /// [`Self::method`]: #method.method
            options: Method::Options,
            /// A shortcut of `self.method(Method::Trace, middleware)`, see [`Self::method`].
            ///
            /// [`Self::method`]: #method.method
            trace: Method::Trace,
            /// A shortcut of `self.method(Method::Patch, middleware)`, see [`Self::method`].
            ///
            /// [`Self::method`]: #method.method
            patch: Method::Patch,
            => $ret
        }
    };
}

macro_rules! impl_methods {
    ($(#[$outer:meta])* $func_name: ident : $methods: expr) => {
        $(#[$outer])*
        pub fn $func_name<M: Middleware<Ex> + 'static>(self, middleware: M) -> Self {
            self.methods($methods, middleware)
        }
    };

    ($($(#[$outer:meta])* $func_name: ident : $methods: expr),* $(,)?) => {
        $(impl_methods!{$(#[$outer])* $func_name: $methods})+
    };
}

/// The middleware for request diversion by HTTP method.
///
/// ## Examples
///
/// ```
/// # use amiya::{middleware::MethodRouter, m};
/// // Other HTTP methods that are not set will set resp to `405 Method Not Allowed`.
/// let router = MethodRouter::new()
///     .get(m!(ctx => ctx.resp.set_body("GET method");))
///     .post(m!(ctx => ctx.resp.set_body("POST method");));
/// ```
///
/// You can set same middleware for different methods by using [`methods`] method.
///
/// ```
/// # use amiya::{middleware::MethodRouter, m, Method};
/// let router = MethodRouter::new()
///     .methods([Method::Get, Method::Post], m!(ctx => ctx.resp.set_body("Hello World");));
/// ```
///
/// [`methods`]: #method.methods
pub struct MethodRouter<Ex> {
    table: HashMap<Method, Arc<dyn Middleware<Ex>>>,
}

impl<Ex> Default for MethodRouter<Ex> {
    fn default() -> Self {
        Self { table: HashMap::new() }
    }
}

impl<Ex> Debug for MethodRouter<Ex> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Method Router { ")?;
        for method in self.table.keys() {
            <Method as Debug>::fmt(method, f)?;
            f.write_str(" ")?;
        }
        f.write_str("}")
    }
}

impl<Ex> MethodRouter<Ex> {
    /// Create a new MethodRouter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set given `middleware` as the handler of specific HTTP method when request hit this router.
    pub fn method<M: Middleware<Ex> + 'static>(mut self, method: Method, middleware: M) -> Self {
        let middleware: Arc<dyn Middleware<Ex>> = Arc::new(middleware);
        self.table.insert(method, Arc::clone(&middleware));
        self
    }

    /// Set given `middleware` as the handler of several HTTP methods when request hit this router.
    pub fn methods<H: AsRef<[Method]>, M: Middleware<Ex> + 'static>(
        mut self, methods: H, middleware: M,
    ) -> Self {
        let middleware: Arc<dyn Middleware<Ex>> = Arc::new(middleware);
        methods.as_ref().iter().for_each(|method| {
            self.table.insert(*method, Arc::clone(&middleware));
        });
        self
    }

    impl_all_http_method! { Self }

    impl_methods! {
        /// Set given `middleware` as the handler of all HTTP method, this method is almost useless
        /// because in this case you can use that `middleware` directly and do not need a
        /// MethodRouter.
        all: ALL_METHODS,
    }
}

#[async_trait]
impl<Ex> Middleware<Ex> for MethodRouter<Ex>
where
    Ex: Send + Sync + 'static,
{
    async fn handle(&self, ctx: Context<'_, Ex>) -> Result {
        if let Some(middleware) = self.table.get(&ctx.req.method()) {
            middleware.handle(ctx).await
        } else {
            ctx.resp.set_status(StatusCode::MethodNotAllowed);
            ctx.resp.take_body();
            Ok(())
        }
    }
}
