//! Лабораторная работа №1 — обработка массивов.
//!
//! 1. [`zero_odd_indices`] — замена элементов с нечётным индексом на ноль.
//! 2. [`Matrix::column_sums`] — сумма элементов каждого столбца матрицы M×N.
//! 3. [`shift_around_max_odd`] — поиск наибольшего нечётного элемента и
//!    циклические сдвиги частей массива слева и справа от него.

use std::fmt;

// ---------------------------------------------------------------------------
// Задание 1
// ---------------------------------------------------------------------------

/// Заменяет каждый элемент с нечётным индексом (1, 3, 5, …) на ноль.
///
/// Нулём считается значение по умолчанию типа (`T::default()`), которое для
/// всех числовых типов Rust равно `0`.
///
/// ```
/// let mut a = [5, 7, 9, 11, 13];
/// lb1::zero_odd_indices(&mut a);
/// assert_eq!(a, [5, 0, 9, 0, 13]);
/// ```
pub fn zero_odd_indices<T: Default>(arr: &mut [T]) {
    for x in arr.iter_mut().skip(1).step_by(2) {
        *x = T::default();
    }
}

// ---------------------------------------------------------------------------
// Задание 2
// ---------------------------------------------------------------------------

/// Ошибки построения матрицы и вычислений над ней.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatrixError {
    /// Строка `row` имеет длину `found`, хотя ожидалось `expected`.
    Ragged {
        row: usize,
        expected: usize,
        found: usize,
    },
    /// Сумма элементов столбца `column` не помещается в `i64`.
    Overflow { column: usize },
}

impl fmt::Display for MatrixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ragged {
                row,
                expected,
                found,
            } => write!(
                f,
                "строка {row} содержит {found} элементов, ожидалось {expected}"
            ),
            Self::Overflow { column } => {
                write!(f, "переполнение при суммировании столбца {column}")
            }
        }
    }
}

impl std::error::Error for MatrixError {}

/// Прямоугольная матрица M×N, хранящаяся построчно в одном векторе.
///
/// Конструктор [`Matrix::from_rows`] гарантирует, что все строки имеют
/// одинаковую длину, поэтому остальные методы не проверяют это повторно.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<i64>,
}

impl Matrix {
    /// Строит матрицу из вектора строк.
    ///
    /// # Errors
    /// [`MatrixError::Ragged`], если строки имеют разную длину.
    pub fn from_rows(rows: Vec<Vec<i64>>) -> Result<Self, MatrixError> {
        let cols = rows.first().map_or(0, Vec::len);
        let mut data = Vec::with_capacity(rows.len() * cols);
        for (i, row) in rows.iter().enumerate() {
            if row.len() != cols {
                return Err(MatrixError::Ragged {
                    row: i,
                    expected: cols,
                    found: row.len(),
                });
            }
            data.extend_from_slice(row);
        }
        Ok(Self {
            rows: rows.len(),
            cols,
            data,
        })
    }

    /// Количество строк M.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Количество столбцов N.
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Итератор по строкам матрицы.
    pub fn iter_rows(&self) -> impl Iterator<Item = &[i64]> {
        // `max(1)` нужен, чтобы `chunks_exact` не паниковал на матрице 0×0.
        self.data.chunks_exact(self.cols.max(1))
    }

    /// Возвращает вектор длины N, где `i`-й элемент — сумма `i`-го столбца.
    ///
    /// # Errors
    /// [`MatrixError::Overflow`], если сумма не помещается в `i64`.
    pub fn column_sums(&self) -> Result<Vec<i64>, MatrixError> {
        let mut sums = vec![0_i64; self.cols];
        for row in self.iter_rows() {
            for (column, (acc, &x)) in sums.iter_mut().zip(row).enumerate() {
                *acc = acc.checked_add(x).ok_or(MatrixError::Overflow { column })?;
            }
        }
        Ok(sums)
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.iter_rows() {
            for x in row {
                write!(f, "{x:>5}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Задание 3
// ---------------------------------------------------------------------------

/// Находит наибольший нечётный элемент и возвращает `(индекс, значение)`.
///
/// Если максимум встречается несколько раз, берётся первое вхождение.
/// Отрицательные числа тоже учитываются: `-3 % 2 == -1 != 0`.
pub fn max_odd(arr: &[i64]) -> Option<(usize, i64)> {
    arr.iter()
        .copied()
        .enumerate()
        .filter(|&(_, x)| x % 2 != 0)
        // `max_by_key` вернул бы последнее вхождение, поэтому сравниваем вручную.
        .fold(None, |best, (i, x)| match best {
            Some((_, b)) if b >= x => best,
            _ => Some((i, x)),
        })
}

/// Один циклический сдвиг влево: `[a, b, c] -> [b, c, a]`.
pub fn cyclic_shift_left_once<T: Copy>(part: &mut [T]) {
    let Some(&first) = part.first() else { return };
    for i in 1..part.len() {
        part[i - 1] = part[i];
    }
    part[part.len() - 1] = first;
}

/// Один циклический сдвиг вправо: `[a, b, c] -> [c, a, b]`.
pub fn cyclic_shift_right_once<T: Copy>(part: &mut [T]) {
    let Some(&last) = part.last() else { return };
    for i in (1..part.len()).rev() {
        part[i] = part[i - 1];
    }
    part[0] = last;
}

/// Выполняет задание 3: находит наибольший нечётный элемент, трижды
/// циклически сдвигает влево элементы справа от него и один раз сдвигает
/// вправо элементы слева от него. Сам максимум остаётся на месте.
///
/// Возвращает найденные `(индекс, значение)` или `None`, если нечётных
/// элементов нет (массив при этом не меняется).
///
/// ```
/// let mut a = [2, 4, 6, 9, 1, 3, 5, 8];
/// assert_eq!(lb1::shift_around_max_odd(&mut a), Some((3, 9)));
/// assert_eq!(a, [6, 2, 4, 9, 8, 1, 3, 5]);
/// ```
pub fn shift_around_max_odd(arr: &mut [i64]) -> Option<(usize, i64)> {
    let (pos, value) = max_odd(arr)?;
    let (left, rest) = arr.split_at_mut(pos);
    let right = &mut rest[1..];

    for _ in 0..3 {
        cyclic_shift_left_once(right);
    }
    cyclic_shift_right_once(left);

    Some((pos, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn zero_odd_basic() {
        let mut a = [1, 2, 3, 4, 5, 6];
        zero_odd_indices(&mut a);
        assert_eq!(a, [1, 0, 3, 0, 5, 0]);
    }

    #[test]
    fn zero_odd_empty_and_single() {
        let mut empty: [i32; 0] = [];
        zero_odd_indices(&mut empty);
        let mut one = [42];
        zero_odd_indices(&mut one);
        assert_eq!(one, [42]);
    }

    #[test]
    fn zero_odd_works_for_floats() {
        let mut a = [1.5, 2.5, 3.5];
        zero_odd_indices(&mut a);
        assert_eq!(a, [1.5, 0.0, 3.5]);
    }

    #[test]
    fn column_sums_basic() {
        let m = Matrix::from_rows(vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]).unwrap();
        assert_eq!(m.column_sums().unwrap(), vec![12, 15, 18]);
    }

    #[test]
    fn column_sums_rectangular() {
        let m = Matrix::from_rows(vec![vec![1, -1, 10, 0], vec![2, -2, 20, 0]]).unwrap();
        assert_eq!((m.rows(), m.cols()), (2, 4));
        assert_eq!(m.column_sums().unwrap(), vec![3, -3, 30, 0]);
    }

    #[test]
    fn ragged_matrix_is_rejected() {
        let err = Matrix::from_rows(vec![vec![1, 2], vec![3]]).unwrap_err();
        assert_eq!(
            err,
            MatrixError::Ragged {
                row: 1,
                expected: 2,
                found: 1
            }
        );
    }

    #[test]
    fn column_sum_overflow_is_reported() {
        let m = Matrix::from_rows(vec![vec![0, i64::MAX], vec![0, 1]]).unwrap();
        assert_eq!(m.column_sums(), Err(MatrixError::Overflow { column: 1 }));
    }

    #[test]
    fn empty_matrix() {
        let m = Matrix::from_rows(vec![]).unwrap();
        assert_eq!(m.column_sums().unwrap(), Vec::<i64>::new());
    }

    #[test]
    fn max_odd_picks_first_maximum_and_negatives() {
        assert_eq!(max_odd(&[2, 7, 4, 7]), Some((1, 7)));
        assert_eq!(max_odd(&[-8, -3, -5]), Some((1, -3)));
        assert_eq!(max_odd(&[2, 4, 6]), None);
        assert_eq!(max_odd(&[]), None);
    }

    #[test]
    fn single_shifts() {
        let mut a = [1, 2, 3, 4];
        cyclic_shift_left_once(&mut a);
        assert_eq!(a, [2, 3, 4, 1]);
        cyclic_shift_right_once(&mut a);
        assert_eq!(a, [1, 2, 3, 4]);
    }

    #[test]
    fn shift_example_from_docs() {
        let mut a = [2, 4, 6, 9, 1, 3, 5, 8];
        assert_eq!(shift_around_max_odd(&mut a), Some((3, 9)));
        assert_eq!(a, [6, 2, 4, 9, 8, 1, 3, 5]);
    }

    #[test]
    fn shift_with_max_at_edges() {
        let mut a = [9, 1, 2, 3, 4];
        shift_around_max_odd(&mut a);
        assert_eq!(a, [9, 4, 1, 2, 3]);

        let mut b = [1, 2, 3, 4, 9];
        shift_around_max_odd(&mut b);
        assert_eq!(b, [4, 1, 2, 3, 9]);
    }

    #[test]
    fn shift_right_part_shorter_than_three() {
        // Три сдвига части длины 2 эквивалентны одному.
        let mut a = [0, 11, 5, 6];
        shift_around_max_odd(&mut a);
        assert_eq!(a, [0, 11, 6, 5]);
    }

    #[test]
    fn no_odd_leaves_array_untouched() {
        let mut a = [2, 4, 6];
        assert_eq!(shift_around_max_odd(&mut a), None);
        assert_eq!(a, [2, 4, 6]);
    }

    proptest! {
        #[test]
        fn prop_zero_odd(v in proptest::collection::vec(any::<i64>(), 0..64)) {
            let mut a = v.clone();
            zero_odd_indices(&mut a);
            for (i, (&before, &after)) in v.iter().zip(&a).enumerate() {
                if i % 2 == 1 { prop_assert_eq!(after, 0); } else { prop_assert_eq!(after, before); }
            }
        }

        #[test]
        fn prop_column_sums_add_up_to_total(
            (rows, cols) in (0usize..8, 0usize..8),
            seed in proptest::collection::vec(-1000i64..1000, 64),
        ) {
            let data: Vec<Vec<i64>> = (0..rows)
                .map(|r| (0..cols).map(|c| seed[r * 8 + c]).collect())
                .collect();
            let m = Matrix::from_rows(data.clone()).unwrap();
            let sums = m.column_sums().unwrap();
            let total: i64 = data.iter().flatten().sum();
            prop_assert_eq!(sums.iter().sum::<i64>(), total);
        }

        #[test]
        fn prop_shifts_match_std_rotate(v in proptest::collection::vec(-50i64..50, 0..40)) {
            let mut actual = v.clone();
            let found = shift_around_max_odd(&mut actual);

            let mut expected = v.clone();
            if let Some((pos, _)) = max_odd(&v) {
                let (left, rest) = expected.split_at_mut(pos);
                let right = &mut rest[1..];
                if !right.is_empty() { let k = 3 % right.len(); right.rotate_left(k); }
                if !left.is_empty() { left.rotate_right(1); }
            }
            prop_assert_eq!(found, max_odd(&v));
            prop_assert_eq!(actual, expected);
        }
    }
}
