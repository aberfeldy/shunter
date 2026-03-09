//! # Source
//!
//! The [`Source`] type is the entry point for building a Shunter pipeline.
//!
//! A pipeline is constructed by chaining stages onto a `Source`, then executed
//! by calling [`Source::run`] with a sink function.
//!
//! ## Stages
//!
//! - [`Source::map`] — transform each element
//! - [`Source::filter`] — drop elements that fail a predicate
//! - [`Source::tap`] — observe elements without modifying them
//! - [`Source::run`] — execute the pipeline, passing results to a sink
//!
//! ## Example
//!
//! ```rust
//! use shunter::source::Source;
//!
//! # #[tokio::main]
//! # async fn main() {
//! Source::new(vec![1, 3, 2, 4])
//!     .map(|x| x * 2)
//!     .filter(|x| *x > 4)
//!     .tap(|x| println!("passing: {}", x))
//!     .run(|x| async move { println!("sink: {}", x) })
//!     .await;
//! # }
//! ```

use std::future::Future;

pub struct Source<T, F> {
    data: Vec<T>,
    func: F,
}

impl<T> Source<T, fn(T) -> Option<T>> {
    pub fn new(data: Vec<T>) -> Self {
        Source { data, func: Some }
    }
}

impl<T, F> Source<T, F> {
    pub fn map<G, U>(self, mut g: G) -> Source<T, impl FnMut(T) -> Option<U>>
    where
        F: FnMut(T) -> Option<T>,
        G: FnMut(T) -> U,
    {
        let mut f = self.func;

        Source {
            data: self.data,
            func: move |x| {
                let y = f(x);
                y.map(&mut g)
            },
        }
    }

    pub fn map_async<G, U, Fut, V>(self, mut g: G) -> Source<T, impl FnMut(T) -> Option<Fut>>
    where
        F: FnMut(T) -> Option<V>,
        G: FnMut(V) -> Fut,
        Fut: Future<Output = U>,
    {
        let mut f = self.func;

        Source {
            data: self.data,
            func: move |x| {
                let y = f(x);
                y.map(&mut g)
            },
        }
    }

    pub fn filter<G, U>(self, mut g: G) -> Source<T, impl FnMut(T) -> Option<U>>
    where
        F: FnMut(T) -> Option<U>,
        G: FnMut(&U) -> bool,
    {
        let mut f = self.func;
        Source {
            data: self.data,
            func: move |x| {
                let y = f(x);
                y.and_then(|v| if g(&v) { Some(v) } else { None })
            },
        }
    }

    pub fn tap<G>(self, mut g: G) -> Source<T, impl FnMut(T) -> Option<T>>
    where
        F: FnMut(T) -> Option<T>,
        G: FnMut(&T),
    {
        let mut f = self.func;

        Source {
            data: self.data,
            func: move |x| {
                let y = f(x);
                y.inspect(|v| g(v))
            },
        }
    }

    pub async fn run<S, Fut, O>(mut self, mut sink: S)
    where
        F: FnMut(T) -> Option<O>,
        S: FnMut(O) -> Fut,
        Fut: Future<Output = ()>,
    {
        for item in self.data {
            if let Some(out) = (self.func)(item) {
                sink(out).await;
            }
        }
    }
}
