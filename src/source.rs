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
//! - [`Source::map_async`] — transform each element asynchronously
//! - [`Source::buffer`] — set max_concurrency
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

/// Internal settings controlling pipeline execution.
///
/// Currently only tracks max concurrency for buffered execution.
struct ExecutionSettings {
    max_concurrency: usize,
}

/// The entry point for building a Shunter pipeline.
///
/// `Source` wraps an input collection and chains transformations
/// that are applied when the pipeline runs. Each stage returns a new
/// `Source` with the composed function.
///
/// # Type Parameters
///
/// - `T`: The input element type
/// - `F`: The composed transformation function
///
/// # Example
///
/// ```
/// use shunter::source::Source;
///
/// let pipeline = Source::new(vec![1, 2, 3])
///     .map(|x| x * 2)
///     .filter(|x| *x > 2);
/// ```
pub struct Source<T, F> {
    data: Vec<T>,
    func: F,
    settings: ExecutionSettings,
}

impl<T> Source<T, fn(T) -> Option<T>> {
    /// Creates a new pipeline from any iterable.
    ///
    /// The initial stage is an identity function that passes all elements through.
    ///
    /// # Example
    ///
    /// ```
    /// use shunter::source::Source;
    ///
    /// let source = Source::new(vec![1, 2, 3]);
    /// ```
    pub fn new<I>(input: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let input = input.into_iter();

        Source {
            data: input.collect(),
            func: Some,
            settings: ExecutionSettings { max_concurrency: 1 },
        }
    }
}

impl<T, F> Source<T, F> {
    /// Transforms each element using the provided function.
    ///
    /// # Example
    ///
    /// ```
    /// use shunter::source::Source;
    ///
    /// let pipeline = Source::new(vec![1, 2, 3]).map(|x| x * 2);
    /// ```
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
            settings: self.settings,
        }
    }

    /// Like `map`, but for async transformations.
    ///
    /// The returned future is stored and executed when the pipeline runs.
    ///
    /// # Example
    ///
    /// ```
    /// use shunter::source::Source;
    ///
    /// let pipeline = Source::new(vec![1, 2, 3])
    ///     .map_async(|x| async move { x * 2 });
    /// ```
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
            settings: self.settings,
        }
    }

    /// Drops elements that don't satisfy the predicate.
    ///
    /// Elements passing the filter continue downstream; others are discarded.
    ///
    /// # Example
    ///
    /// ```
    /// use shunter::source::Source;
    ///
    /// let pipeline = Source::new(vec![1, 2, 3, 4])
    ///     .filter(|x| *x > 2); // keeps 3, 4
    /// ```
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
            settings: self.settings,
        }
    }

    /// Observes elements without modifying them.
    ///
    /// Useful for logging, debugging, or side effects. The element passes through unchanged.
    ///
    /// # Example
    ///
    /// ```
    /// use shunter::source::Source;
    ///
    /// let pipeline = Source::new(vec![1, 2, 3])
    ///     .tap(|x| println!("processing: {}", x));
    /// ```
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
            settings: self.settings,
        }
    }

    /// Sets the max number of elements to process concurrently.
    ///
    /// Defaults to 1 (sequential execution).
    ///
    /// # Example
    ///
    /// ```
    /// use shunter::source::Source;
    ///
    /// let pipeline = Source::new(vec![1, 2, 3, 4, 5])
    ///     .buffer(3); // process up to 3 elements at a time
    /// ```
    pub fn buffer(self, len: usize) -> Source<T, F> {
        let new_settings = ExecutionSettings {
            max_concurrency: len,
        };
        Source {
            settings: new_settings,
            ..self
        }
    }

    /// Executes the pipeline and sends results to the sink.
    ///
    /// Processes elements through the composed function, buffering up to
    /// `max_concurrency` elements. When the buffer is full, one buffered
    /// element is sent to the sink before continuing.
    /// Remaining elements are flushed at the end.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use shunter::source::Source;
    ///
    /// Source::new(vec![1, 2, 3])
    ///     .map(|x| x * 2)
    ///     .run(|x| async move { println!("{}", x) })
    ///     .await;
    /// ```
    pub async fn run<S, Fut, O>(mut self, mut sink: S)
    where
        F: FnMut(T) -> Option<O>,
        S: FnMut(O) -> Fut,
        Fut: Future<Output = ()>,
    {
        let mut queue = vec![];
        for item in self.data {
            if let Some(out) = (self.func)(item) {
                queue.push(out);
                if queue.len() >= self.settings.max_concurrency {
                    if let Some(concurrent) = queue.pop() {
                        sink(concurrent).await
                    }
                }
            }
        }

        while !queue.is_empty() {
            if let Some(out) = queue.pop() {
                sink(out).await
            }
        }
    }
}
