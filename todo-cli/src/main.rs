use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct Zadanie {
    id: usize,                    // уникальный идентификатор
    text: String,                 //текст задачи
    done: bool,                   //статус выполнения
    created_at: String,           // дата создания
    completed_at: Option<String>, //дата выполнения (может отсутствовать)
}
// Методы для структкры Zadanie
impl Zadanie {
    fn new(id: usize, text: String) -> Self {
        Zadanie {
            id,
            text,
            done: false,
            created_at: Local::now().to_string(),
            completed_at: None,
        }
    }
    fn complete(&mut self) {
        self.done = true;
        self.completed_at = Some(Local::now().to_string());
    }
}
struct ToDolist {
    zadaniey: Vec<Zadanie>, // Хранение задач
    next_id: usize,         // Счётчик для новых ID
}
impl ToDolist {
    fn new() -> Self {
        ToDolist {
            zadaniey: Vec::new(),
            next_id: 1,
        }
    }

    // Добавление задачи
    fn add(&mut self, text: String) {
        let zadanie = Zadanie::new(self.next_id, text);
        self.zadaniey.push(zadanie);
        self.next_id += 1;
        println!("✅ Задача добавлена (Номер задачи = {})", self.next_id - 1)
    }
}
// ОТОБРАЖЕНИЕ ЗАДАЧ
impl ToDolist {
    fn list(&self) {
        if self.zadaniey.is_empty() {
            println!("📭 Список задач пуст");
            return;
        }
        println!("📋 Список задач:");
        for zadanie in &self.zadaniey {
            let status = if zadanie.done { "✓" } else { " " };
            println!("{:3} [{}] {}", zadanie.id, status, zadanie.text);
        }
    }
}
// ДОБАВЛЕНИЕ ОПЕРАЦИ С ЗАДАЧАМИ
impl ToDolist {
    fn complete(&mut self, id: usize) -> Result<(), String> {
        match self.zadaniey.iter_mut().find(|t| t.id == id) {
            Some(zadanie) => {
                zadanie.complete();
                Ok(println!("👍 Задача {} выполнена", id))
            }
            None => Err(format!("❌ Задача с Номером {} не найдена", id)),
        }
    }

    // УДАЛЕНИЕ ЗАДАЧИ
    fn remove(&mut self, id: usize) -> Result<(), String> {
        let index = self.zadaniey.iter().position(|t| t.id == id);
        match index {
            Some(i) => {
                self.zadaniey.remove(i);
                Ok(println!("🗑️ Задача {} удалена", id))
            }
            None => Err(format!("❌ Задача с ID {} не найдена", id)),
        }
    }
}
// РЕАЛИЗУЕМ СОХРАНЕНИЕ И ЗАГРУЗКУ
// СОХРАНЕНИЕ
fn save_to_file(zadaniey: &[Zadanie]) -> io::Result<()> {
    let json = serde_json::to_string(zadaniey)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("zadaniey.json")?;
    file.write_all(json.as_bytes())?;
    Ok(())
}
//ЗАГРУЗКA
fn load_from_file() -> io::Result<Vec<Zadanie>> {
    if !Path::new("zadaniey.json").exists() {
        return Ok(Vec::new());
    }
    let mut file = File::open("zadaniey.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let zadaniey = serde_json::from_str(&contents)?;
    Ok(zadaniey)
}

// ОСНОВНОЙ ЦИКЛ ПРОГРАММЫ
fn main() {
    //Загрузка задач
    let mut todo = match load_from_file() {
        Ok(zadaniey) => {
            let next_id = zadaniey.iter().map(|t| t.id).max().unwrap_or(0) + 1;
            ToDolist { zadaniey, next_id }
        }
        Err(e) => {
            eprintln!("⚠️ Ошибка загрузки: {}. Новый список создан.", e);
            ToDolist::new()
        }
    };
    //Основной цикл
    loop {
        println!("\n📌 Команды: добавить, список, выполнить, удалить, выход");
        print!("➥");
        io::Write::flush(&mut io::stdout()).unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let parts: Vec<&str> = input.trim().splitn(2, ' ').collect();

        match parts[0] {
            "добавить" if parts.len() > 1 => {
                todo.add(parts[1].to_string());
                save_to_file(&todo.zadaniey).unwrap();
            }
            "список" => todo.list(),
            "выполнить" if parts.len() > 1 => {
                if let Ok(id) = parts[1].parse::<usize>() {
                    if let Err(e) = todo.complete(id) {
                        eprintln!("{}", e);
                    } else {
                        save_to_file(&todo.zadaniey).unwrap();
                    }
                } else {
                    eprintln!("❌ Неверный ID задачи");
                }
            }
            "удалить" if parts.len() > 1 => {
                if let Ok(id) = parts[1].parse::<usize>() {
                    if let Err(e) = todo.remove(id) {
                        eprintln!("{}", e);
                    } else {
                        save_to_file(&todo.zadaniey).unwrap();
                    }
                } else {
                    eprintln!("❌ Неверный ID задачи");
                }
            }
            "выход" => break,
            _ => println!("❌ Неизвестная команда. Попробуйте снова."),
        }
    }
}
