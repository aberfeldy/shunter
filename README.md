# Shunter

**Shunter** is an experimental streaming pipeline library for Rust.

It provides a small DSL for building composable data pipelines inspired by systems like Akka Streams, but implemented in
an idiomatic Rust style.

The goal is to define clear, readable pipelines from a **Source** to a **Sink** while keeping stages easily composable.

⚠️ **Status:** Early **Alpha** — APIs will change.

---

# Example

```rust
Source::new(vec![1, 3, 2, 4])
.map( | x| x * 2)
.filter( | x| x > 4)
.tap( | x| println!("passing: {}", x))
.run(sink)
.await;
```

Conceptually this builds a pipeline:

Source → map → filter → tap → sink

Each stage is composed into a single pipeline function that processes the stream elements.

---

# Current Features

Phase 1 functionality currently includes:

## Source

Create a pipeline from a collection.

```rust
Source::new(Vec<T>)
```

## map

Transform items.

```rust
.map( | x| transform(x))
```

Example:

```rust
.map( | x| x * 2)
```

## filter

Conditionally remove items.

```rust
.filter( | x| predicate(x))
```

Example:

```rust
.filter( | x| x > 10)
```

Items failing the predicate are skipped.

## tap

Observe values without modifying them.

Useful for logging, metrics, debugging, etc.

```rust
.tap( | x| println!("{}", x))
```

## run

Execute the pipeline and send the results to a sink.

```rust
.run(sink)
```

The sink receives each element produced by the pipeline.

---

# Pipeline Model

Internally, Shunter composes stages into a single function:

T → Option<U\>

This allows stages like `filter` to drop elements while keeping the pipeline simple.

Stages operate conceptually like:

Option\<A\> → Option\<B\>

This ensures that skipped items propagate through the pipeline cleanly.

---

# Design Goals

Shunter aims to provide:

- Composable pipeline DSL
- Simple stage model
- Minimal runtime overhead
- Clear Source → Stage → Sink structure

Future versions will expand this toward more advanced stream processing features.

---

# Planned Features

Planned stages and capabilities include:

- async stages
- buffered processing
- parallel map
- fan-out / broadcast
- merge / zip
- error handling
- observability hooks

These are **not implemented yet**.

---

# Status

Shunter is currently in **early alpha**.

Working:

- Source
- map
- filter
- tap
- run

Everything else is still under development.

Expect:

- API changes
- refactors
- missing features
