use lb3::{Queue, Stack};

/// Проверка сбалансированности скобок — классическая задача на стек.
fn brackets_balanced(s: &str) -> bool {
    let mut stack = Stack::new();
    for c in s.chars() {
        match c {
            '(' | '[' | '{' => stack.push(c),
            ')' | ']' | '}' => {
                let expected = match c {
                    ')' => '(',
                    ']' => '[',
                    _ => '{',
                };
                if stack.pop() != Some(expected) {
                    return false;
                }
            }
            _ => {}
        }
    }
    stack.is_empty()
}

fn main() {
    println!("=== Задание 1. Очередь (FIFO) ===");
    let mut q = Queue::with_capacity(4);
    for client in ["Анна", "Борис", "Вера"] {
        q.enqueue(client);
        println!("enqueue({client:<5}) -> {q:?}");
    }
    println!("peek()          -> {:?}", q.peek());
    println!("dequeue()       -> {:?}, очередь {q:?}", q.dequeue());
    for client in ["Глеб", "Дина", "Егор"] {
        q.enqueue(client);
        println!(
            "enqueue({client:<5}) -> {q:?} (ёмкость буфера {})",
            q.capacity()
        );
    }
    while let Some(client) = q.dequeue() {
        println!("dequeue()       -> {client:<5} осталось {}", q.len());
    }
    println!("peek() пустой   -> {:?}", q.peek());

    println!();
    println!("=== Задание 2. Стек (LIFO) ===");
    let mut s = Stack::new();
    for x in [10, 20, 30] {
        s.push(x);
        println!("push({x})  -> {s:?}");
    }
    println!("peek()    -> {:?}", s.peek());
    println!("pop()     -> {:?}, стек {s:?}", s.pop());
    println!("pop()     -> {:?}, стек {s:?}", s.pop());
    println!("pop()     -> {:?}, стек {s:?}", s.pop());
    println!("pop()     -> {:?} (стек пуст)", s.pop());

    println!();
    println!("Применение стека: проверка скобок");
    for expr in [
        "{[()()]}",
        "fn main() { let v = vec![1, 2]; }",
        "([)]",
        "(()",
    ] {
        let verdict = if brackets_balanced(expr) {
            "сбалансированы"
        } else {
            "НЕ сбалансированы"
        };
        println!("  {expr:<36} -> {verdict}");
    }
}
