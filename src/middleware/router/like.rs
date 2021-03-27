use {crate::Middleware, std::borrow::Cow};

#[doc(hidden)]
pub trait RouterLike<Ex>: Sized {
    fn set_endpoint<M: Middleware<Ex> + 'static>(&mut self, middleware: M);
    fn set_fallback<M: Middleware<Ex> + 'static>(&mut self, middleware: M);
    fn insert_to_router_table<P, M>(&mut self, path: P, middleware: M)
    where
        P: Into<Cow<'static, str>>,
        M: Middleware<Ex> + 'static;
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
