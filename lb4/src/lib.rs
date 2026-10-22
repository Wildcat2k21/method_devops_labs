//! Лабораторная работа №4 — конкурентные вычисления.
//!
//! В Go каждая функция запускается как горутина (`go f(i)`), а результаты
//! собираются через канал. В Rust показаны два эквивалента:
//!
//! * [`run_threads`] — потоки ОС (`std::thread`) + канал `std::sync::mpsc`;
//! * [`run_tasks`] — асинхронные задачи `tokio` (лёгкие задачи на пуле
//!   рабочих потоков, модель M:N — ближе всего к горутинам).

use std::fmt;
use std::io::BufRead;
use std::sync::mpsc;
use std::thread::{self, ThreadId};

/// Наибольший `n`, для которого `F(n)` помещается в `u128`.
pub const MAX_FIBONACCI: u32 = 186;
/// Наибольший `n`, для которого `n!` помещается в `u128`.
pub const MAX_FACTORIAL: u32 = 34;

/// Результат не помещается в `u128`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Overflow;

impl fmt::Display for Overflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("переполнение u128")
    }
}

impl std::error::Error for Overflow {}

/// `n`-е число Фибоначчи: `F(0) = 0`, `F(1) = F(2) = 1`, `F(n) = F(n-1) + F(n-2)`.
///
/// ```
/// assert_eq!(lb4::fibonacci(10), Ok(55));
/// ```
pub fn fibonacci(n: u32) -> Result<u128, Overflow> {
    if n == 0 {
        return Ok(0);
    }
    // Инвариант: a = F(k-1), b = F(k). Не вычисляем F(n+1), чтобы
    // F(186) — последнее помещающееся в u128 число — не давало переполнения.
    let (mut a, mut b) = (0_u128, 1_u128);
    for _ in 1..n {
        let next = a.checked_add(b).ok_or(Overflow)?;
        a = b;
        b = next;
    }
    Ok(b)
}

/// Факториал `n! = 1 · 2 · … · n`, `0! = 1`.
///
/// ```
/// assert_eq!(lb4::factorial(10), Ok(3_628_800));
/// ```
pub fn factorial(n: u32) -> Result<u128, Overflow> {
    (1..=u128::from(n)).try_fold(1_u128, |acc, k| acc.checked_mul(k).ok_or(Overflow))
}

/// Какая функция вычислялась.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    Fibonacci,
    Factorial,
}

impl Kind {
    pub fn compute(self, n: u32) -> Result<u128, Overflow> {
        match self {
            Self::Fibonacci => fibonacci(n),
            Self::Factorial => factorial(n),
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // `pad` учитывает ширину и выравнивание из форматной строки.
        f.pad(match self {
            Self::Fibonacci => "fib",
            Self::Factorial => "fact",
        })
    }
}

/// Результат одной конкурентной задачи.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// Порядковый номер входного значения (для сортировки результатов).
    pub index: usize,
    pub kind: Kind,
    pub n: u32,
    pub value: Result<u128, Overflow>,
    /// Имя потока ОС, на котором выполнялось вычисление.
    pub worker: String,
    /// Идентификатор этого потока (имена рабочих потоков могут совпадать).
    pub thread_id: ThreadId,
}

fn current_worker() -> (String, ThreadId) {
    let t = thread::current();
    (t.name().unwrap_or("<без имени>").to_owned(), t.id())
}

/// Запускает по два потока ОС на каждое входное число (Фибоначчи и
/// факториал). Потоки отправляют результаты в канал; функция возвращает
/// их в порядке *поступления*, который зависит от планировщика ОС.
pub fn run_threads(inputs: &[u32]) -> Vec<Outcome> {
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::with_capacity(inputs.len() * 2);

    for (index, &n) in inputs.iter().enumerate() {
        for kind in [Kind::Fibonacci, Kind::Factorial] {
            let tx = tx.clone();
            let handle = thread::Builder::new()
                .name(format!("{kind}-{n}"))
                .spawn(move || {
                    let value = kind.compute(n);
                    let (worker, thread_id) = current_worker();
                    // Ошибка отправки возможна, только если приёмник уже
                    // уничтожен, — здесь он живёт до конца функции.
                    let _ = tx.send(Outcome {
                        index,
                        kind,
                        n,
                        value,
                        worker,
                        thread_id,
                    });
                })
                .expect("не удалось создать поток");
            handles.push(handle);
        }
    }
    // Закрываем исходный отправитель, чтобы цикл приёма завершился,
    // когда отработают все потоки (аналог close(ch) в Go).
    drop(tx);

    let outcomes: Vec<Outcome> = rx.iter().collect();
    for h in handles {
        h.join().expect("поток завершился паникой");
    }
    outcomes
}

/// То же самое на асинхронных задачах `tokio`: тысячи задач могут
/// выполняться на нескольких рабочих потоках рантайма.
pub async fn run_tasks(inputs: &[u32]) -> Vec<Outcome> {
    let mut set = tokio::task::JoinSet::new();
    for (index, &n) in inputs.iter().enumerate() {
        for kind in [Kind::Fibonacci, Kind::Factorial] {
            set.spawn(async move {
                let (worker, thread_id) = current_worker();
                Outcome {
                    index,
                    kind,
                    n,
                    value: kind.compute(n),
                    worker,
                    thread_id,
                }
            });
        }
    }
    let mut outcomes = Vec::with_capacity(set.len());
    while let Some(res) = set.join_next().await {
        outcomes.push(res.expect("задача завершилась паникой"));
    }
    outcomes
}

/// Ошибка чтения входных чисел.
#[derive(Debug)]
pub enum InputError {
    Io(std::io::Error),
    /// Ввод закончился раньше, чем было прочитано нужное количество чисел.
    UnexpectedEof {
        got: usize,
        expected: usize,
    },
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "ошибка ввода-вывода: {e}"),
            Self::UnexpectedEof { got, expected } => {
                write!(f, "ввод закончился: прочитано {got} из {expected} чисел")
            }
        }
    }
}

impl std::error::Error for InputError {}

/// Читает `count` неотрицательных целых чисел из `reader`.
///
/// Числа могут быть разделены пробелами и/или переводами строк. Некорректные
/// токены не прерывают ввод: для каждого вызывается `on_invalid`, и чтение
/// продолжается.
///
/// # Errors
/// [`InputError::UnexpectedEof`], если ввод закончился раньше времени.
pub fn read_numbers<R: BufRead>(
    reader: R,
    count: usize,
    mut on_invalid: impl FnMut(&str),
) -> Result<Vec<u32>, InputError> {
    let mut numbers = Vec::with_capacity(count);
    for line in reader.lines() {
        let line = line.map_err(InputError::Io)?;
        for token in line.split_whitespace() {
            match token.parse::<u32>() {
                Ok(n) => numbers.push(n),
                Err(_) => on_invalid(token),
            }
            if numbers.len() == count {
                return Ok(numbers);
            }
        }
    }
    Err(InputError::UnexpectedEof {
        got: numbers.len(),
        expected: count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn fibonacci_first_values() {
        let got: Vec<u128> = (0..=10).map(|n| fibonacci(n).unwrap()).collect();
        assert_eq!(got, [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);
    }

    #[test]
    fn factorial_first_values() {
        let got: Vec<u128> = (0..=10).map(|n| factorial(n).unwrap()).collect();
        assert_eq!(
            got,
            [1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362_880, 3_628_800]
        );
    }

    #[test]
    fn overflow_boundaries() {
        assert!(fibonacci(MAX_FIBONACCI).is_ok());
        assert_eq!(fibonacci(MAX_FIBONACCI + 1), Err(Overflow));
        assert!(factorial(MAX_FACTORIAL).is_ok());
        assert_eq!(factorial(MAX_FACTORIAL + 1), Err(Overflow));
    }

    fn sorted(mut v: Vec<Outcome>) -> Vec<(usize, Kind, u32, Result<u128, Overflow>)> {
        v.sort_by_key(|o| (o.index, o.kind));
        v.into_iter()
            .map(|o| (o.index, o.kind, o.n, o.value))
            .collect()
    }

    fn expected(inputs: &[u32]) -> Vec<(usize, Kind, u32, Result<u128, Overflow>)> {
        inputs
            .iter()
            .enumerate()
            .flat_map(|(i, &n)| [Kind::Fibonacci, Kind::Factorial].map(|k| (i, k, n, k.compute(n))))
            .collect()
    }

    #[test]
    fn threads_compute_everything() {
        let inputs: Vec<u32> = (1..=10).collect();
        let out = run_threads(&inputs);
        assert_eq!(out.len(), 20);
        // Каждое вычисление выполнялось в своём именованном потоке.
        let workers: HashSet<_> = out.iter().map(|o| o.thread_id).collect();
        assert_eq!(workers.len(), 20);
        assert_eq!(sorted(out), expected(&inputs));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn tasks_compute_everything() {
        let inputs = [5, 0, 40, 200, 7, 7];
        let out = run_tasks(&inputs).await;
        // 12 задач выполнились максимум на 2 рабочих потоках (модель M:N).
        let workers: HashSet<_> = out.iter().map(|o| o.thread_id).collect();
        assert!(workers.len() <= 2, "{workers:?}");
        assert_eq!(sorted(out), expected(&inputs));
    }

    #[test]
    fn read_numbers_skips_garbage() {
        let input = "1 2 abc 3\n\n-4 5\n6 7 8 9 10 11\n";
        let mut invalid = Vec::new();
        let nums = read_numbers(input.as_bytes(), 10, |t| invalid.push(t.to_owned())).unwrap();
        assert_eq!(nums, [1, 2, 3, 5, 6, 7, 8, 9, 10, 11]);
        assert_eq!(invalid, ["abc", "-4"]);
    }

    #[test]
    fn read_numbers_reports_eof() {
        let err = read_numbers("1 2 3".as_bytes(), 10, |_| {}).unwrap_err();
        assert!(matches!(
            err,
            InputError::UnexpectedEof {
                got: 3,
                expected: 10
            }
        ));
    }
}
