use lb1::{Matrix, shift_around_max_odd, zero_odd_indices};

fn main() {
    println!("=== Задание 1. Замена элементов с нечётным индексом на ноль ===");
    let mut arr = [12, -7, 3, 18, 25, 4, 9, 30, 11];
    println!("Исходный массив:  {arr:?}");
    zero_odd_indices(&mut arr);
    println!("Результат:        {arr:?}");

    println!();
    println!("=== Задание 2. Сумма элементов каждого столбца ===");
    let matrix = Matrix::from_rows(vec![
        vec![3, 8, -1, 4, 10],
        vec![5, 0, 7, -2, 6],
        vec![9, 1, 2, 3, -4],
        vec![-6, 11, 4, 8, 1],
    ])
    .expect("все строки одинаковой длины");
    println!("Матрица {}×{}:", matrix.rows(), matrix.cols());
    print!("{matrix}");
    match matrix.column_sums() {
        Ok(sums) => {
            println!("{}", "-".repeat(matrix.cols() * 5));
            let line: String = sums.iter().map(|s| format!("{s:>5}")).collect();
            println!("{line}  <- суммы столбцов");
        }
        Err(e) => eprintln!("Ошибка: {e}"),
    }

    println!();
    println!("=== Задание 3. Сдвиги вокруг наибольшего нечётного элемента ===");
    let mut arr = [4, 15, 8, 2, 21, 6, 13, 10, 7, 12];
    println!("Исходный массив:  {arr:?}");
    match shift_around_max_odd(&mut arr) {
        Some((pos, value)) => {
            println!("Наибольший нечётный элемент: {value} (индекс {pos})");
            println!("Слева  от него: 1 циклический сдвиг вправо");
            println!("Справа от него: 3 циклических сдвига влево");
            println!("Результат:        {arr:?}");
        }
        None => println!("В массиве нет нечётных элементов"),
    }
}
