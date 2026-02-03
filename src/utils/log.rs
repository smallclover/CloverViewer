// debug 构建时，真正调用 tracing
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! dev_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*);
    };
}

// release 构建时，什么都不做（编译期抹除）
#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! dev_info {
    ($($arg:tt)*) => {};
}
