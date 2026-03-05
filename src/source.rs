use std::future::Future;

pub struct Source<T, F> {
    data: Vec<T>,
    func: F,
}

impl<T> Source<T, fn(T) -> T> {
    pub fn new(data: Vec<T>) -> Self {
        fn identity<T>(x: T) -> T {
            x
        }
        Source {
            data,
            func: identity::<T>,
        }
    }
}

impl<T, F> Source<T, F> {
    pub async fn run<S, Fut, O>(mut self, mut sink: S)
    where
        F: FnMut(T) -> O,
        S: FnMut(O) -> Fut,
        Fut: Future<Output = ()>,
    {
        for item in self.data {
            let out = (self.func)(item);
            sink(out).await;
        }
    }
}
