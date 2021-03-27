use {
    crate::{async_trait, Context, Middleware, Result},
    std::{future::Future, pin::Pin},
};

type BoxedResultFut<'x> = Pin<Box<dyn Future<Output = Result> + Send + 'x>>;

/// The wrapper for use async function or closure as a middleware.
///
/// This is the type when you use macro [`m`] , **Do Not** use this type directly!
///
/// [`m`]: ../macro.m.html
#[allow(missing_debug_implementations)]
pub struct M<Ex> {
    /// the code in macro [`m`], converted to a boxed async func.
    ///
    /// **Do Not** set this field by hand, use macro [`m`] instead!
    ///
    /// [`m`]: ../macro.m.html
    pub func: Box<dyn Fn(Context<'_, Ex>) -> BoxedResultFut<'_> + Send + Sync>,
}

#[async_trait]
impl<Ex> Middleware<Ex> for M<Ex>
where
    Ex: Send + Sync + 'static,
{
    async fn handle(&self, ctx: Context<'_, Ex>) -> Result<()> {
        (self.func)(ctx).await
    }
}

/// Writer middleware easily.
///
/// It's a macro to let you easily write middleware use closure and syntax like JavaScript's
/// arrow function, or convert a async fn to a middleware use the `m!(async_func_name)` syntax.
///
/// It returns a [`M`] instance, which implement [`Middleware`] trait.
///
/// ## Examples
///
/// ### Convert a async function to middleware
///
/// ```
/// # use amiya::{Context, Result, m};
/// async fn response(mut ctx: Context<'_, ()>) -> Result {
///     ctx.next().await?;
///     ctx.resp.set_body("Hello world");
///     Ok(())
/// }
///
/// let app = amiya::new().uses(m!(response));
/// ```
///
/// ### Convert a block to middleware
///
/// Syntax: `<Context parameter name> [: Extra data type] => { <your code> }`
///
/// Default extra data type is `()`, same bellow.
///
/// ```
/// # use amiya::m;
/// //                                 | this `: ()` can be omitted
/// //                                 v
/// let app = amiya::new().uses(m!(ctx: () => {
///     ctx.next().await?;
///     ctx.resp.set_body("Hello world");
///     Ok(())
/// }));
/// ```
///
/// ### Convert a expr to middleware
///
/// Syntax: `<Context parameter name> [: Extra data type] => <The expr>`
///
/// ```
/// # use amiya::{Context, Result, m};
/// async fn response(msg: &'static str, mut ctx: Context<'_, ()>) -> Result {
///     ctx.next().await?;
///     ctx.resp.set_body(msg);
///     Ok(())
/// }
///
/// let app = amiya::new().uses(m!(ctx => response("Hello World", ctx).await));
/// ```
///
/// ### Convert statements to middleware
///
/// Syntax: `<Context parameter name> [: Extra data type] => <statements>`
///
/// ```
/// # use amiya::m;
/// let app = amiya::new().uses(m!(ctx => ctx.resp.set_body("Hello World");));
/// ```
///
/// Notice you do not return a value here, because a `Ok(())` is auto added.
///
/// This is expand to:
///
/// ```text
/// ctx.resp.set_body("Hello World");
/// Ok(())
/// ```
///
/// [`M`]: middleware/struct.M.html
/// [`Middleware`]: middleware/trait.Middleware.html
#[macro_export]
macro_rules! m {
    // Convert a async function to middleware by function name

    ($func: ident) => {
        $crate::middleware::M { func: Box::new(|ctx| Box::pin($func(ctx))) }
    };

    // Convert a block

    ($ctx: ident : $ex: ty => $body: block ) => {
        $crate::middleware::M {
            func: Box::new(move |mut $ctx: $crate::Context<'_, $ex>| {
                Box::pin(async move { $body })
            }),
        }
    };

    ($ctx: ident => $body: block) => {
        m!($ctx: () => $body)
    };

    // Convert one expr

    ($ctx: ident : $ex: ty => $body: expr) => {
        m!($ctx: $ex => { $body })
    };

    ($ctx: ident => $body: expr) => {
        m!($ctx => { $body })
    };

    // Convert statements

    ($ctx: ident : $ex: ty => $($body: tt)+) => {
        m!($ctx: $ex => { $($body)+ ; Ok(()) })
    };

    ($ctx: ident => $($body: tt)+) => {
        m!($ctx => { $($body)+ ; Ok(()) })
    };
}
