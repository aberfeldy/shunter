use std::future::Future;

pub struct Source<T, F> {
    data: Vec<T>,
    func: F,
}

impl<T> Source<T, fn(T) -> Option<T>> {
    pub fn new(data: Vec<T>) -> Self {
        Source {
            data,
            func: Some,
        }
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
    
    pub fn map_async<G, U, Fut, V>(
        self,
        mut g: G,
    ) -> Source<T, impl FnMut(T) -> Option<Fut>>
    where
        F: FnMut(T) -> Option<V>,
        G: FnMut(V) -> Fut,
        Fut: Future<Output=U>,
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
                y.inspect(|v| {
                    g(v)
                })
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
