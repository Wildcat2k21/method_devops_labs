//! Лабораторная работа №3 — структуры данных «очередь» и «стек».
//!
//! * [`Queue`] — FIFO-очередь на кольцевом буфере
//!   (`enqueue` / `dequeue` / `peek` за амортизированное O(1)).
//! * [`Stack`] — LIFO-стек на динамическом массиве
//!   (`push` / `pop` / `peek` за амортизированное O(1)).

pub mod queue;
pub mod stack;

pub use queue::Queue;
pub use stack::Stack;
