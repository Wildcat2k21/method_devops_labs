//! Лабораторная работа №2 — рекурсия и операции над массивом.
//!
//! 1. [`sum_recursive`] — рекурсивный подсчёт суммы элементов
//!    (и [`sum_divide_and_conquer`] — вариант с глубиной рекурсии O(log N)).
//! 2. [`insert_at`] — вставка элемента по индексу `k`.
//! 3. [`remove_at`] — удаление элемента по индексу `k`.

use std::fmt;

// ---------------------------------------------------------------------------
// Задание 1
// ---------------------------------------------------------------------------

/// Рекурсивная сумма: `sum([a, rest..]) = a + sum(rest)`, `sum([]) = 0`.
///
/// Глубина рекурсии равна длине массива, поэтому на очень больших
/// массивах (сотни тысяч элементов) возможно переполнение стека —
/// для них есть [`sum_divide_and_conquer`].
///
/// ```
/// assert_eq!(lb2::sum_recursive(&[1, 2, 3, 4]), 10);
/// ```
pub fn sum_recursive(arr: &[i64]) -> i64 {
    match arr {
        [] => 0,
        [first, rest @ ..] => first + sum_recursive(rest),
    }
}

/// Рекурсивная сумма «разделяй и властвуй»: массив делится пополам,
/// суммы половин складываются. Глубина рекурсии — `⌈log2 N⌉`.
///
/// ```
/// let big: Vec<i64> = (1..=1_000_000).collect();
/// assert_eq!(lb2::sum_divide_and_conquer(&big), 500_000_500_000);
/// ```
pub fn sum_divide_and_conquer(arr: &[i64]) -> i64 {
    match arr {
        [] => 0,
        [x] => *x,
        _ => {
            let (left, right) = arr.split_at(arr.len() / 2);
            sum_divide_and_conquer(left) + sum_divide_and_conquer(right)
        }
    }
}

// ---------------------------------------------------------------------------
// Задания 2 и 3
// ---------------------------------------------------------------------------

/// Индекс вне допустимого диапазона.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexError {
    pub index: usize,
    pub len: usize,
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "индекс {} вне диапазона для массива длины {}",
            self.index, self.len
        )
    }
}

impl std::error::Error for IndexError {}

/// Вставляет `value` в позицию `k`, сдвигая хвост массива на один вправо.
///
/// Допустимы индексы `0..=len`: `k == len` означает вставку в конец.
///
/// # Errors
/// [`IndexError`], если `k > len`; массив при этом не меняется.
///
/// ```
/// let mut v = vec![10, 20, 40];
/// lb2::insert_at(&mut v, 2, 30).unwrap();
/// assert_eq!(v, [10, 20, 30, 40]);
/// ```
pub fn insert_at<T>(arr: &mut Vec<T>, k: usize, value: T) -> Result<(), IndexError> {
    let len = arr.len();
    if k > len {
        return Err(IndexError { index: k, len });
    }
    // Кладём новый элемент в конец и «протаскиваем» его влево до позиции k,
    // попутно сдвигая элементы [k..len) на одну позицию вправо.
    arr.push(value);
    for i in (k..len).rev() {
        arr.swap(i, i + 1);
    }
    Ok(())
}

/// Удаляет элемент с индексом `k`, сдвигая хвост массива на один влево,
/// и возвращает удалённый элемент.
///
/// # Errors
/// [`IndexError`], если `k >= len`; массив при этом не меняется.
///
/// ```
/// let mut v = vec![10, 20, 30, 40];
/// assert_eq!(lb2::remove_at(&mut v, 1), Ok(20));
/// assert_eq!(v, [10, 30, 40]);
/// ```
pub fn remove_at<T>(arr: &mut Vec<T>, k: usize) -> Result<T, IndexError> {
    let len = arr.len();
    if k >= len {
        return Err(IndexError { index: k, len });
    }
    // Перемещаем удаляемый элемент в конец, сдвигая [k+1..len) влево.
    for i in k..len - 1 {
        arr.swap(i, i + 1);
    }
    Ok(arr.pop().expect("массив не пуст: k < len"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn sum_small_arrays() {
        assert_eq!(sum_recursive(&[]), 0);
        assert_eq!(sum_recursive(&[7]), 7);
        assert_eq!(sum_recursive(&[3, -5, 10, 0, 2]), 10);
        assert_eq!(sum_divide_and_conquer(&[3, -5, 10, 0, 2]), 10);
    }

    #[test]
    fn insert_into_start_middle_end() {
        let mut v = vec![2, 3];
        insert_at(&mut v, 0, 1).unwrap();
        assert_eq!(v, [1, 2, 3]);
        insert_at(&mut v, 3, 5).unwrap();
        assert_eq!(v, [1, 2, 3, 5]);
        insert_at(&mut v, 3, 4).unwrap();
        assert_eq!(v, [1, 2, 3, 4, 5]);
    }

    #[test]
    fn insert_into_empty() {
        let mut v: Vec<String> = Vec::new();
        insert_at(&mut v, 0, "a".to_owned()).unwrap();
        assert_eq!(v, ["a"]);
    }

    #[test]
    fn insert_out_of_range() {
        let mut v = vec![1, 2, 3];
        assert_eq!(
            insert_at(&mut v, 4, 9),
            Err(IndexError { index: 4, len: 3 })
        );
        assert_eq!(v, [1, 2, 3]);
    }

    #[test]
    fn remove_from_start_middle_end() {
        let mut v = vec![1, 2, 3, 4, 5];
        assert_eq!(remove_at(&mut v, 0), Ok(1));
        assert_eq!(remove_at(&mut v, 3), Ok(5));
        assert_eq!(remove_at(&mut v, 1), Ok(3));
        assert_eq!(v, [2, 4]);
    }

    #[test]
    fn remove_out_of_range() {
        let mut v: Vec<i32> = vec![];
        assert_eq!(remove_at(&mut v, 0), Err(IndexError { index: 0, len: 0 }));
        let mut v = vec![1];
        assert_eq!(remove_at(&mut v, 1), Err(IndexError { index: 1, len: 1 }));
    }

    proptest! {
        #[test]
        fn prop_sums_match_iterator(v in proptest::collection::vec(-1_000_000i64..1_000_000, 0..500)) {
            let expected: i64 = v.iter().sum();
            prop_assert_eq!(sum_recursive(&v), expected);
            prop_assert_eq!(sum_divide_and_conquer(&v), expected);
        }

        #[test]
        fn prop_insert_matches_std(v in proptest::collection::vec(any::<i32>(), 0..50), k in 0usize..60, x: i32) {
            let mut actual = v.clone();
            let mut expected = v.clone();
            let res = insert_at(&mut actual, k, x);
            if k <= v.len() {
                expected.insert(k, x);
                prop_assert!(res.is_ok());
            } else {
                prop_assert!(res.is_err());
            }
            prop_assert_eq!(actual, expected);
        }

        #[test]
        fn prop_remove_matches_std(v in proptest::collection::vec(any::<i32>(), 0..50), k in 0usize..60) {
            let mut actual = v.clone();
            let mut expected = v.clone();
            let res = remove_at(&mut actual, k);
            if k < v.len() {
                prop_assert_eq!(res, Ok(expected.remove(k)));
            } else {
                prop_assert!(res.is_err());
            }
            prop_assert_eq!(actual, expected);
        }

        #[test]
        fn prop_insert_then_remove_is_identity(v in proptest::collection::vec(any::<i32>(), 0..50), k in 0usize..50, x: i32) {
            let k = k.min(v.len());
            let mut w = v.clone();
            insert_at(&mut w, k, x).unwrap();
            prop_assert_eq!(remove_at(&mut w, k), Ok(x));
            prop_assert_eq!(w, v);
        }
    }
}
