use std::collections::BTreeSet;
use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};

use lb4::{Kind, Outcome, read_numbers, run_tasks, run_threads};

const USAGE: &str = "\
Использование:
  lb4 task1   — Фибоначчи и факториал для i = 1..10 (потоки и задачи tokio)
  lb4 task2   — прочитать 10 чисел с консоли и обработать их конкурентно";

fn print_arrival(title: &str, outcomes: &[Outcome]) {
    println!("{title}:");
    for (pos, o) in outcomes.iter().enumerate() {
        let value = match o.value {
            Ok(v) => v.to_string(),
            Err(e) => format!("ошибка: {e}"),
        };
        println!(
            "  {:>2}. {:<4}({:>3}) = {:<40} [{}]",
            pos + 1,
            o.kind,
            o.n,
            value,
            o.worker
        );
    }
}

fn print_table(outcomes: &[Outcome]) {
    let mut rows: Vec<&Outcome> = outcomes.iter().collect();
    rows.sort_by_key(|o| (o.index, o.kind));
    println!("Итоговая таблица (отсортирована по входу):");
    println!("  {:>4} | {:>40} | {:>40}", "n", "fib(n)", "n!");
    println!("  {}", "-".repeat(4 + 3 + 40 + 3 + 40));
    for pair in rows.chunks(2) {
        let cell = |kind: Kind| {
            let o = pair.iter().find(|o| o.kind == kind).expect("обе функции");
            match o.value {
                Ok(v) => v.to_string(),
                Err(e) => e.to_string(),
            }
        };
        println!(
            "  {:>4} | {:>40} | {:>40}",
            pair[0].n,
            cell(Kind::Fibonacci),
            cell(Kind::Factorial)
        );
    }
}

fn run_all(inputs: &[u32]) {
    println!("--- Вариант A: потоки ОС (std::thread) + канал mpsc ---");
    let outcomes = run_threads(inputs);
    print_arrival("Порядок поступления результатов в канал", &outcomes);
    println!();
    print_table(&outcomes);

    println!();
    println!("--- Вариант B: асинхронные задачи tokio (M:N, аналог горутин) ---");
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name_fn(|| {
            static NEXT: AtomicUsize = AtomicUsize::new(1);
            format!("tokio-worker-{}", NEXT.fetch_add(1, Ordering::Relaxed))
        })
        .build()
        .expect("не удалось создать рантайм tokio");
    let outcomes = runtime.block_on(run_tasks(inputs));
    print_arrival("Порядок завершения задач (JoinSet::join_next)", &outcomes);
    let workers: BTreeSet<_> = outcomes.iter().map(|o| o.worker.as_str()).collect();
    println!();
    println!(
        "Задач: {}; рабочих потоков рантайма, выполнивших их: {} {workers:?}",
        outcomes.len(),
        workers.len()
    );
}

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        Some("task1") => {
            println!("=== Задание 1. Горутины в цикле от 1 до 10 ===");
            let inputs: Vec<u32> = (1..=10).collect();
            run_all(&inputs);
            ExitCode::SUCCESS
        }
        Some("task2") => {
            println!("=== Задание 2. Обработка 10 чисел, введённых с консоли ===");
            print!("Введите 10 неотрицательных целых чисел через пробел или Enter: ");
            io::stdout().flush().ok();
            let stdin = io::stdin();
            if !stdin.is_terminal() {
                // При вводе из конвейера Enter не нажимается — переносим строку сами.
                println!();
            }
            let stdin = stdin.lock();
            match read_numbers(stdin, 10, |bad| {
                eprintln!("  пропущено «{bad}»: ожидалось целое число ≥ 0");
            }) {
                Ok(inputs) => {
                    println!();
                    println!("Считанный срез: {inputs:?}");
                    println!();
                    run_all(&inputs);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Ошибка: {e}");
                    ExitCode::FAILURE
                }
            }
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}
