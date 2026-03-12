use shunter::source::Source;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

type BoxFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;

fn test_sink<T>() -> (Arc<Mutex<Vec<T>>>, impl FnMut(T) -> BoxFuture<()>)
where
    T: Send + 'static,
{
    let out = Arc::new(Mutex::new(Vec::new()));
    let arc = out.clone();

    let sink = move |item: T| {
        let arc = arc.clone();
        Box::pin(async move {
            arc.lock().unwrap().push(item);
        }) as BoxFuture<()>
    };

    (out, sink)
}

#[tokio::test]
async fn it_runs_simple_stage() {
    let (out, sink) = test_sink();
    Source::new(vec![1, 3, 2, 4]).run(sink).await;
    assert_eq!(*out.lock().unwrap(), vec![1, 3, 2, 4]);
}

#[tokio::test]
async fn it_runs_with_simple_map() {
    let (out, sink) = test_sink();
    Source::new(vec![1, 3, 2, 4]).map(|x| x * 2).run(sink).await;
    assert_eq!(*out.lock().unwrap(), vec![2, 6, 4, 8]);
}

#[tokio::test]
async fn it_runs_with_type_map() {
    let (out, sink) = test_sink();
    Source::new(vec![1, 3, 2, 4])
        .map(|x| x * 2)
        .map(|x| x.to_string())
        .run(sink)
        .await;
    assert_eq!(*out.lock().unwrap(), vec!["2", "6", "4", "8"]);
}
#[tokio::test]
async fn it_runs_with_filter() {
    let (out, sink) = test_sink();
    Source::new(vec![1, 3, 2, 4])
        .filter(|x| *x > 2)
        .run(sink)
        .await;
    assert_eq!(*out.lock().unwrap(), vec![3, 4]);
}
#[tokio::test]
async fn it_runs_with_filter_and_map() {
    let (out, sink) = test_sink();
    Source::new(vec![1, 3, 2, 4])
        .filter(|x| *x > 2)
        .map(|x| x * 2)
        .run(sink)
        .await;
    assert_eq!(*out.lock().unwrap(), vec![6, 8]);
}

#[tokio::test]
async fn it_runs_with_tap() {
    let (out, sink) = test_sink();
    Source::new(vec![1, 3, 2, 4])
        .tap(|x| println!("{}", *x))
        .run(sink)
        .await;
    assert_eq!(*out.lock().unwrap(), vec![1, 3, 2, 4]);
}

#[tokio::test]
async fn it_runs_with_tap_sf() {
    let mut log: Vec<i8> = Vec::new();
    let (out, sink) = test_sink();
    Source::new(vec![1, 3, 2, 4])
        .tap(|x| log.push(*x))
        .run(sink)
        .await;
    assert_eq!(*out.lock().unwrap(), vec![1, 3, 2, 4]);
    assert_eq!(log, vec![1, 3, 2, 4]);
}
#[tokio::test]
async fn it_runs_with_map_async() {
    let out = Arc::new(Mutex::new(Vec::new()));
    let arc = out.clone();
    let sink = move |item| {
        let arc = arc.clone();
        Box::pin(async move {
            let value = item.await;
            arc.lock().unwrap().push(value);
        }) as BoxFuture<()>
    };

    Source::new(vec![1, 3, 2, 4])
        .map_async(|x| async move { x * 2 })
        .run(sink)
        .await;
    assert_eq!(*out.lock().unwrap(), vec![2, 6, 4, 8]);
}

#[tokio::test]
async fn it_creates_with_iterator() {
    let (out, sink) = test_sink();
    Source::new([1, 3, 2, 4]).run(sink).await;
    assert_eq!(*out.lock().unwrap(), vec![1, 3, 2, 4]);

    let (out, sink) = test_sink();
    Source::new([1, 3, 2, 4].into_iter()).run(sink).await;
    assert_eq!(*out.lock().unwrap(), vec![1, 3, 2, 4]);

    let (out, sink) = test_sink();
    Source::new(1..5).run(sink).await;
    assert_eq!(*out.lock().unwrap(), vec![1, 2, 3, 4]);
}

#[tokio::test]
async fn buffer_parallel_processing() {
    let out = Arc::new(Mutex::new(Vec::new()));
    let arc = out.clone();
    let sink = move |item| {
        let arc = arc.clone();
        Box::pin(async move {
            let value = item.await;
            arc.lock().unwrap().push(value);
        }) as BoxFuture<()>
    };

    Source::new(vec![1, 3, 2, 4])
        .map_async(|x| async move { x * 2 })
        .buffer(2)
        .run(sink)
        .await;
    let i: HashSet<_> = vec![2, 4, 6, 8].into_iter().collect();

    let o: HashSet<_> = out
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .collect::<HashSet<_>>();

    assert_eq!(i, o);
}
