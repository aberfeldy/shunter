[![Crates.io](https://img.shields.io/crates/v/shunter)](https://crates.io/crates/shunter)
[![Docs.rs](https://docs.rs/shunter/badge.svg)](https://docs.rs/shunter)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

# Shunter

**Shunter** is an experimental streaming pipeline library for Rust.

## v0.1.1: What's New?

- **Buffer support** — new method to control how many elements get processed concurrently. When the buffer is full, one
  buffered element is sent to the sink before continuing.
- **Collect** — gather all pipeline results into a Vec instead of streaming to a sink.

See the [buffer](#buffer) and [collect](#collect) sections below for details.

It provides a small DSL for building composable data pipelines inspired by stream processing systems, implemented in an
idiomatic Rust style.

The goal is to define clear, readable pipelines from a **Source** to a **Sink** while keeping stages easily composable.

⚠️ **Status:** Early **Alpha** — APIs will change.

---

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
shunter = "0.1.1"
```

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

## map_async

Transform items asynchronously.

Example:

```rust
.map( | x| async move { x * 2 }))
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

## buffer

Control concurrent processing. When the buffer is full, one buffered element is sent to the sink before continuing.

```rust
.buffer(5)  // process up to 5 elements at a time
```

## collect

Gather all pipeline results into a Vec. Useful when you want the output as a collection instead of streaming to a sink.

```rust
let results: Vec<i32> = Source::new(vec![1, 2, 3])
    .map(|x| x * 2)
    .collect()
    .await;
```

## run

Execute the pipeline and send results to a sink.

```rust
.run(|x| async move { println!("{}", x) })
```

The sink receives each element. If you've set a buffer size, elements get sent when the buffer fills, with any remaining
flushed at the end.

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
- buffer
- run
- collect

Everything else is still under development.

Expect:

- API changes
- refactors
- missing features
