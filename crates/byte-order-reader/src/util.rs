#[macro_export]
macro_rules! ready_ok {
    ($e:expr $(,)?) => {
        match $e {
            core::task::Poll::Ready(t) => match t {
                core::result::Result::Ok(t) => t,
                core::result::Result::Err(e) => {
                    return core::task::Poll::Ready(core::result::Result::Err(e.into()))
                }
            },
            core::task::Poll::Pending => return core::task::Poll::Pending,
        }
    };
}
