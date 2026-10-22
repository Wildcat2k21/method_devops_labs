//! Очередь (FIFO) на кольцевом буфере.

use std::fmt;

const MIN_CAPACITY: usize = 4;

/// Очередь «первым пришёл — первым ушёл».
///
/// Элементы хранятся в кольцевом буфере `buf`: `head` указывает на начало
/// очереди, следующий свободный слот — `(head + len) % capacity`. Когда буфер
/// заполнен, ёмкость удваивается, а элементы переписываются по порядку.
///
/// ```
/// use lb3::Queue;
///
/// let mut q = Queue::new();
/// q.enqueue("a");
/// q.enqueue("b");
/// assert_eq!(q.peek(), Some(&"a"));
/// assert_eq!(q.dequeue(), Some("a"));
/// assert_eq!(q.dequeue(), Some("b"));
/// assert_eq!(q.dequeue(), None);
/// ```
pub struct Queue<T> {
    buf: Vec<Option<T>>,
    head: usize,
    len: usize,
}

impl<T> Queue<T> {
    /// Создаёт пустую очередь без выделения памяти.
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            head: 0,
            len: 0,
        }
    }

    /// Создаёт пустую очередь с заранее выделенным буфером.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: std::iter::repeat_with(|| None).take(capacity).collect(),
            head: 0,
            len: 0,
        }
    }

    /// Добавляет элемент в конец очереди.
    pub fn enqueue(&mut self, value: T) {
        if self.len == self.buf.len() {
            self.grow();
        }
        let tail = (self.head + self.len) % self.buf.len();
        self.buf[tail] = Some(value);
        self.len += 1;
    }

    /// Забирает элемент из начала очереди.
    pub fn dequeue(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let value = self.buf[self.head].take();
        self.head = (self.head + 1) % self.buf.len();
        self.len -= 1;
        value
    }

    /// Возвращает ссылку на следующий элемент, не извлекая его.
    pub fn peek(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            self.buf[self.head].as_ref()
        }
    }

    /// Количество элементов в очереди.
    pub fn len(&self) -> usize {
        self.len
    }

    /// `true`, если очередь пуста.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Текущая ёмкость кольцевого буфера.
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    /// Итератор по элементам от начала к концу очереди.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let cap = self.buf.len();
        (0..self.len).filter_map(move |i| self.buf[(self.head + i) % cap].as_ref())
    }

    fn grow(&mut self) {
        let new_cap = (self.buf.len() * 2).max(MIN_CAPACITY);
        let mut new_buf: Vec<Option<T>> = Vec::with_capacity(new_cap);
        // Переносим элементы в порядке очереди, начиная с head.
        for i in 0..self.len {
            let idx = (self.head + i) % self.buf.len();
            new_buf.push(self.buf[idx].take());
        }
        new_buf.resize_with(new_cap, || None);
        self.buf = new_buf;
        self.head = 0;
    }
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> FromIterator<T> for Queue<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut q = Self::new();
        for x in iter {
            q.enqueue(x);
        }
        q
    }
}

impl<T: fmt::Debug> fmt::Debug for Queue<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("front ")?;
        f.debug_list().entries(self.iter()).finish()?;
        f.write_str(" back")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::VecDeque;

    #[test]
    fn empty_queue() {
        let mut q: Queue<i32> = Queue::new();
        assert!(q.is_empty());
        assert_eq!(q.peek(), None);
        assert_eq!(q.dequeue(), None);
    }

    #[test]
    fn fifo_order() {
        let mut q = Queue::new();
        for i in 1..=5 {
            q.enqueue(i);
        }
        assert_eq!(q.len(), 5);
        assert_eq!(q.peek(), Some(&1));
        let drained: Vec<_> = std::iter::from_fn(|| q.dequeue()).collect();
        assert_eq!(drained, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn wraps_around_and_grows() {
        let mut q = Queue::with_capacity(4);
        q.enqueue(1);
        q.enqueue(2);
        q.enqueue(3);
        assert_eq!(q.dequeue(), Some(1));
        assert_eq!(q.dequeue(), Some(2));
        // head = 2: следующие элементы «заворачивают» в начало буфера.
        q.enqueue(4);
        q.enqueue(5);
        q.enqueue(6);
        assert_eq!(q.capacity(), 4);
        // Буфер полон — следующая вставка удваивает ёмкость.
        q.enqueue(7);
        assert_eq!(q.capacity(), 8);
        assert_eq!(q.iter().copied().collect::<Vec<_>>(), [3, 4, 5, 6, 7]);
    }

    #[test]
    fn works_with_non_copy_types() {
        let mut q: Queue<String> = ["x", "y"].into_iter().map(String::from).collect();
        assert_eq!(q.peek().map(String::as_str), Some("x"));
        assert_eq!(q.dequeue().as_deref(), Some("x"));
        assert_eq!(format!("{q:?}"), r#"front ["y"] back"#);
    }

    #[derive(Debug, Clone)]
    enum Op {
        Enqueue(i32),
        Dequeue,
        Peek,
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            any::<i32>().prop_map(Op::Enqueue),
            Just(Op::Dequeue),
            Just(Op::Peek),
        ]
    }

    proptest! {
        /// Модельное тестирование: поведение совпадает с `VecDeque`.
        #[test]
        fn prop_behaves_like_vecdeque(ops in proptest::collection::vec(op(), 0..200)) {
            let mut q = Queue::new();
            let mut model = VecDeque::new();
            for op in ops {
                match op {
                    Op::Enqueue(x) => { q.enqueue(x); model.push_back(x); }
                    Op::Dequeue => prop_assert_eq!(q.dequeue(), model.pop_front()),
                    Op::Peek => prop_assert_eq!(q.peek(), model.front()),
                }
                prop_assert_eq!(q.len(), model.len());
            }
            prop_assert!(q.iter().eq(model.iter()));
        }
    }
}
