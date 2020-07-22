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
        /// This method is auto generated, it's a proxy of `self.method($func_name, middleware)`
        /// So, see `method` method of this type for document.
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
            get: Method::Get,
            head: Method::Head,
            post: Method::Post,
            put: Method::Put,
            delete: Method::Delete,
            connect: Method::Connect,
            options: Method::Options,
            trace: Method::Trace,
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

/// MethodRouter middleware for request diversion by HTTP method
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
    /// Set given `middleware` as the handler of specific HTTP method when request hit this router
    pub fn method<M: Middleware<Ex> + 'static>(mut self, method: Method, middleware: M) -> Self {
        let middleware: Arc<dyn Middleware<Ex>> = Arc::new(middleware);
        self.table.insert(method, Arc::clone(&middleware));
        self
    }

    /// Set given `middleware` as the handler of different HTTP methods when request hit this router
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
        /// because in this case you can simply use that `middleware` and do not need a MethodRouter
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
