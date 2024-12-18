pub trait Capture<T> {}
impl<T, R> Capture<T> for R where R: ?Sized {}