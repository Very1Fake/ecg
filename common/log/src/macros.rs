pub use tracing;

#[cfg(feature = "tracy")]
pub use tracy_client;

#[cfg(not(feature = "tracy"))]
#[macro_export]
macro_rules! span {
    ($guard:tt, $level:ident, $label:expr, $($fields:tt)*) => {
        let span = $crate::tracing::span!(tracing::Level::$level, $label, $($fields)*);
        let $guard = span.enter();
    };
    ($guard:tt, $level:ident, $label:expr) => {
        let span = $crate::tracing::span!(tracing::Level::$level, $label);
        let $guard = span.enter();
    };
    ($guard:tt, $label:expr) => {
        let span = $crate::tracing::span!($crate::tracing::Level::TRACE, $label);
        let $guard = span.enter();
    };
    ($guard:tt, $no_tracy_label:expr, $tracy_label:expr) => {
        $crate::span!($guard, $no_tracy_label);
    };
}

#[cfg(feature = "tracy")]
#[macro_export]
macro_rules! span {
    ($guard:tt, $level:ident, $label:expr, $($fields:tt)*) => {
        let span = tracing::span!(tracing::Level::$level, $label, $($fields)*);
        let $guard = span.enter();
    };
    ($guard:tt, $level:ident, $label:expr) => {
        let span = tracing::span!(tracing::Level::$level, $label);
        let $guard = span.enter();
    };
    ($guard:tt, $label:expr) => {
        $crate::prof_alloc!($guard, $label);
    };
    ($guard:tt, $no_tracy_label:expr, $tracy_label:expr) => {
        $crate::span!($guard, $tracy_label);
    };
}

#[cfg(not(feature = "tracy"))]
pub struct ProfSpan;

#[cfg(not(feature = "tracy"))]
impl Drop for ProfSpan {
    fn drop(&mut self) {}
}

#[cfg(not(feature = "tracy"))]
#[macro_export]
macro_rules! prof {
    ($guard_name:tt, $name:expr) => {
        let $guard_name = $crate::ProfSpan;
    };
    ($name:expr) => {
        $crate::prof!(_guard, $name);
    };
}

#[cfg(not(feature = "tracy"))]
#[macro_export]
macro_rules! prof_alloc {
    ($guard:tt, $label:expr) => {
        let $guard = $crate::ProfSpan;
    };
    ($label:expr) => {
        $crate::prof!(_guard, $label);
    };
}

#[cfg(feature = "tracy")]
pub struct ProfSpan(pub tracy_client::Span);

#[cfg(feature = "tracy")]
#[macro_export]
macro_rules! prof {
    ($guard_name:tt, $name:expr) => {
        let $guard_name = $crate::ProfSpan($crate::tracy_client::span!($name, 0));
    };
    ($name:expr) => {
        $crate::prof!(_guard, $name);
    };
}

#[cfg(feature = "tracy")]
#[macro_export]
macro_rules! prof_alloc {
    ($guard_label:tt, $label:expr) => {
        let $guard_label = $crate::ProfSpan({
            struct T;
            let type_name = core::any::type_name::<T>();
            $crate::tracy_client::Client::running()
                .expect("prof_alloc! without a running tracy_client::Client")
                .span_alloc(
                    Some($label),
                    &type_name[..type_name.len() - 3],
                    file!(),
                    line!(),
                    0,
                )
        });
    };
    ($label:expr) => {
        $crate::prof!(_guard, $label);
    };
}
