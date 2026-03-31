use rusqlite::{Connection, Result};
use std::collections::HashMap;

pub fn analyze_keyboard_usage(db_path: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;
    
    let mut stmt = conn.prepare("SELECT key, count FROM key_presses")?;
    let key_iter = stmt.query_map([], |row| {
        let key: String = row.get(0)?;
        let count: u64 = row.get(1)?;
        Ok((key, count))
    })?;

    let mut counts: HashMap<String, u64> = HashMap::new();
    let mut total: u64 = 0;

    for key in key_iter {
        let (k, v) = key?;
        total += v;
        counts.insert(k, v);
    }

    if total == 0 {
        println!("Нет нажатий для анализа.");
        return Ok(());
    }

    let main_keys = ["KeyA","KeyB","KeyC","KeyD","KeyE","KeyF","KeyG","KeyH","KeyI","KeyJ",
                     "KeyK","KeyL","KeyM","KeyN","KeyO","KeyP","KeyQ","KeyR","KeyS","KeyT",
                     "KeyU","KeyV","KeyW","KeyX","KeyY","KeyZ",
                     "Digit0","Digit1","Digit2","Digit3","Digit4","Digit5","Digit6","Digit7","Digit8","Digit9",
                     "Space"];
    let mut main_count = 0;
    let mut extra_count = 0;

    for (k, v) in counts.iter() {
        if main_keys.contains(&k.as_str()) {
            main_count += v;
        } else {
            extra_count += v;
        }
    }

    let main_ratio = main_count as f64 / total as f64;

    let recommendation = if main_ratio > 0.8 {
        "40% или 60% клавиатура подходит"
    } else if main_ratio > 0.6 {
        "60% или TKL клавиатура подходит"
    } else {
        "TKL или полноразмерная клавиатура предпочтительна"
    };

    println!("Анализ использования клавиш:");
    println!("Всего нажатий: {}", total);
    println!("Основные клавиши: {} ({:.1}%)", main_count, main_ratio*100.0);
    println!("Дополнительные клавиши: {} ({:.1}%)", extra_count, 100.0-main_ratio*100.0);
    println!("Рекомендация: {}", recommendation);

    Ok(())
}