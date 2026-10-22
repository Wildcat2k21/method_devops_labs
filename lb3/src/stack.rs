//! Стек (LIFO) на динамическом массиве.

use std::fmt;

/// Стек «последним пришёл — первым ушёл».
///
/// Вершина стека — последний элемент вектора `items`, поэтому добавление и
/// извлечение не требуют сдвига остальных элементов.
///
/// ```
/// use lb3::Stack;
///
/// let mut s = Stack::new();
/// s.push(1);
/// s.push(2);
/// assert_eq!(s.peek(), Some(&2));
/// assert_eq!(s.pop(), Some(2));
/// assert_eq!(s.pop(), Some(1));
/// assert_eq!(s.pop(), None);
/// ```
pub struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    /// Создаёт пустой стек.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Кладёт элемент на вершину стека.
    pub fn push(&mut self, value: T) {
        self.items.push(value);
    }

    /// Снимает элемент с вершины стека.
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Возвращает ссылку на вершину стека, не снимая её.
    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    /// Количество элементов в стеке.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// `true`, если стек пуст.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Итератор от вершины к дну стека.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter().rev()
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FromIterator<T> for Stack<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            items: iter.into_iter().collect(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("top ")?;
        f.debug_list().entries(self.iter()).finish()?;
        f.write_str(" bottom")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn empty_stack() {
        let mut s: Stack<i32> = Stack::new();
        assert!(s.is_empty());
        assert_eq!(s.peek(), None);
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn lifo_order() {
        let mut s: Stack<_> = (1..=5).collect();
        assert_eq!(s.len(), 5);
        assert_eq!(s.peek(), Some(&5));
        let drained: Vec<_> = std::iter::from_fn(|| s.pop()).collect();
        assert_eq!(drained, [5, 4, 3, 2, 1]);
    }

    #[test]
    fn debug_shows_top_first() {
        let s: Stack<_> = ['a', 'b', 'c'].into_iter().collect();
        assert_eq!(format!("{s:?}"), "top ['c', 'b', 'a'] bottom");
    }

    #[derive(Debug, Clone)]
    enum Op {
        Push(i32),
        Pop,
        Peek,
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            any::<i32>().prop_map(Op::Push),
            Just(Op::Pop),
            Just(Op::Peek)
        ]
    }

    proptest! {
        /// Модельное тестирование: поведение совпадает с `Vec`.
        #[test]
        fn prop_behaves_like_vec(ops in proptest::collection::vec(op(), 0..200)) {
            let mut s = Stack::new();
            let mut model = Vec::new();
            for op in ops {
                match op {
                    Op::Push(x) => { s.push(x); model.push(x); }
                    Op::Pop => prop_assert_eq!(s.pop(), model.pop()),
                    Op::Peek => prop_assert_eq!(s.peek(), model.last()),
                }
                prop_assert_eq!(s.len(), model.len());
            }
        }
    }
}
