mod analysis;

use rdev::{listen, EventType, Key};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use rusqlite::{params, Connection};
use chrono::Utc;
use std::io::{self, Write};

fn main() {
    println!("Keyboard Analyzer Microservice");
    println!("1. Начать запись клавиш");
    println!("2. Сбросить базу");
    println!("3. Выйти");

    print!("Выберите опцию: ");
    io::stdout().flush().unwrap();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    let choice = choice.trim();

    match choice {
        "1" => start_recording(),
        "2" => reset_database(),
        "3" => {
            println!("Выход...");
            return;
        },
        _ => {
            println!("Неверная опция. Выход...");
            return;
        }
    }
}

fn start_recording() {
    println!("Запись клавиш запущена...");
    println!("Для завершения используйте Ctrl + Alt + Q");

    let conn = Connection::open("key_analyzer.db").expect("Не удалось создать/открыть DB");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS key_presses (
            key TEXT PRIMARY KEY,
            count INTEGER NOT NULL,
            timestamp TEXT NOT NULL
        )",
        [],
    ).unwrap();
    let conn = Arc::new(Mutex::new(conn));
    let key_counts = Arc::new(Mutex::new(HashMap::<String, u64>::new()));

    let (tx, rx) = mpsc::channel();
    let key_counts_clone = key_counts.clone();
    let conn_clone = conn.clone();
    let tx_clone = tx.clone();

    thread::spawn(move || {
        let mut ctrl_pressed = false;
        let mut alt_pressed = false;

        let res = listen(move |event| {
            if let EventType::KeyPress(key) = event.event_type {
                if key == Key::ControlLeft || key == Key::ControlRight { ctrl_pressed = true; }
                if key == Key::Alt { alt_pressed = true; }

                if ctrl_pressed && alt_pressed && key == Key::KeyQ {
                    let _ = tx_clone.send(());
                }

                let key_name = format!("{:?}", key);

                {
                    let mut map = key_counts_clone.lock().unwrap();
                    let counter = map.entry(key_name.clone()).or_insert(0);
                    *counter += 1;
                }

                let timestamp = Utc::now().to_rfc3339();
                let conn_lock = conn_clone.lock().unwrap();
                let _ = conn_lock.execute(
                    "INSERT INTO key_presses (key, count, timestamp)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(key) DO UPDATE SET
                         count = count + excluded.count,
                         timestamp = excluded.timestamp",
                    params![key_name, 1, timestamp],
                );

                println!("Pressed: {}", key_name);
            }

            if let EventType::KeyRelease(key) = event.event_type {
                if key == Key::ControlLeft || key == Key::ControlRight { ctrl_pressed = false; }
                if key == Key::Alt { alt_pressed = false; }
            }
        });

        if let Err(err) = res {
            eprintln!("Error listening to keyboard: {:?}", err);
        }
    });

    rx.recv().unwrap();
    println!("Запись завершена.");

    let map = key_counts.lock().unwrap();
    println!("Итоговые нажатия:");
    for (k, v) in map.iter() {
        println!("{}: {}", k, v);
    }

    if let Err(e) = analysis::analyze_keyboard_usage("key_analyzer.db") {
        eprintln!("Ошибка анализа: {:?}", e);
    }
}

fn reset_database() {
    print!("Вы действительно хотите удалить базу? (y/N): ");
    io::stdout().flush().unwrap();

    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).unwrap();
    let confirm = confirm.trim();

    if confirm.eq_ignore_ascii_case("y") {
        let _ = std::fs::remove_file("key_analyzer.db");
        println!("База данных удалена.");
    } else {
        println!("Сброс отменён.");
    }
}