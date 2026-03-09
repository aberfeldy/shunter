//! # Shunter
//!
//! An experimental streaming pipeline library for Rust.
//!
//! Shunter provides a composable DSL for building data pipelines with a clear
//! Source → Stage → Sink structure, inspired by stream processing systems.
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
//!
//! ## Pipeline Model
//!
//! Stages are composed into a single function `T → Option<U>`, allowing `filter`
//! to drop elements while keeping the pipeline simple and allocation-free.
//!
//! ## Status
//!
//! Early **alpha** — APIs will change.

pub mod source;
