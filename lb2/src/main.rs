use lb2::{insert_at, remove_at, sum_divide_and_conquer, sum_recursive};

fn main() {
    println!("=== Задание 1. Рекурсивная сумма элементов массива ===");
    let arr = [7, -3, 12, 5, 0, 9, -1, 4];
    println!("Массив: {arr:?}");
    println!("sum_recursive          = {}", sum_recursive(&arr));
    println!("sum_divide_and_conquer = {}", sum_divide_and_conquer(&arr));
    let big: Vec<i64> = (1..=1_000_000).collect();
    println!(
        "Сумма 1..=1_000_000 (разделяй и властвуй, глубина ~20) = {}",
        sum_divide_and_conquer(&big)
    );

    println!();
    println!("=== Задание 2. Вставка элемента по индексу k ===");
    let mut v = vec![10, 20, 30, 40, 50];
    println!("Исходный массив: {v:?}");
    for (k, value) in [(2, 25), (0, 5), (7, 60), (10, 99)] {
        match insert_at(&mut v, k, value) {
            Ok(()) => println!("insert_at(k={k}, value={value}) -> {v:?}"),
            Err(e) => println!("insert_at(k={k}, value={value}) -> ошибка: {e}"),
        }
    }

    println!();
    println!("=== Задание 3. Удаление элемента по индексу k ===");
    println!("Исходный массив: {v:?}");
    for k in [0, 3, 5, 42] {
        match remove_at(&mut v, k) {
            Ok(removed) => println!("remove_at(k={k}) -> удалён {removed}, массив {v:?}"),
            Err(e) => println!("remove_at(k={k}) -> ошибка: {e}"),
        }
    }
}
