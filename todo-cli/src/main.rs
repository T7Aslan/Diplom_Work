use chrono::{Local, NaiveDate}; // Работа с датами/временем
use serde::{Deserialize, Serialize}; // Сериализация/десериализация
use std::fs::{File, OpenOptions}; // Работа с файлами
use std::io::{self, Read, Write}; // Ввод/вывод
use std::path::Path; // Работа с путями

/// Структура задачи с автоматической сериализацией
#[derive(Debug, Deserialize, Serialize)]
struct Zadanie {
    id: usize,                    // Уникальный числовой идентификатор
    text: String,                 // Текст задачи
    done: bool,                   // Статус выполнения (true/false)
    created_at: String,           // Дата создания в строковом формате
    completed_at: Option<String>, // Дата выполнения (None если не выполнена)
    due_date: Option<String>,     // Срок выполнения (опционально)
}

impl Zadanie {
    /// Создание новой задачи без срока выполнения
    fn new(id: usize, text: String) -> Self {
        Zadanie {
            id,
            text,
            done: false,                          // По умолчанию не выполнена
            created_at: Local::now().to_string(), // Текущая дата/время
            completed_at: None,                   // Пока нет даты выполнения
            due_date: None,                       // Срок не установлен
        }
    }

    /// Создание задачи с возможным сроком выполнения
    fn with_due_date(id: usize, text: String, due_date: Option<String>) -> Self {
        Zadanie {
            id,
            text,
            done: false,
            created_at: Local::now().to_string(),
            completed_at: None,
            due_date, // Устанавливаем переданный срок
        }
    }

    /// Отметка задачи как выполненной
    fn complete(&mut self) {
        self.done = true;
        self.completed_at = Some(Local::now().to_string()); // Фиксируем время выполнения
    }
}

/// Основной контейнер для работы с задачами
struct ToDolist {
    zadaniey: Vec<Zadanie>, // Динамический массив задач
    next_id: usize,         // Счётчик для генерации новых ID
}

impl ToDolist {
    /// Создание нового пустого списка
    fn new() -> Self {
        ToDolist {
            zadaniey: Vec::new(), // Пустой вектор
            next_id: 1,           // Начинаем с ID = 1
        }
    }

    /// Добавление задачи без срока
    fn add(&mut self, text: String) {
        let zadanie = Zadanie::new(self.next_id, text);
        self.zadaniey.push(zadanie);
        self.next_id += 1; // Увеличиваем счётчик
        println!("✅ Задача добавлена (ID: {})", self.next_id - 1);
    }

    /// Добавление задачи со сроком выполнения
    fn add_with_date(&mut self, text: String, date_str: &str) -> Result<(), String> {
        // Парсим дату в формате ГГГГ-ММ-ДД
        match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            Ok(_) => {
                // Если дата валидна
                let zadanie =
                    Zadanie::with_due_date(self.next_id, text, Some(date_str.to_string()));
                self.zadaniey.push(zadanie);
                self.next_id += 1;
                println!("✅ Задача с сроком добавлена (ID: {})", self.next_id - 1);
                Ok(())
            }
            Err(_) => Err("❌ Неверный формат даты. Используйте ГГГГ-ММ-ДД".to_string()),
        }
    }

    /// Вывод списка задач с прогрессом
    fn list(&self) {
        if self.zadaniey.is_empty() {
            println!("📭 Список задач пуст");
            return;
        }

        // Рассчёт прогресса выполнения
        let total = self.zadaniey.len();
        let done = self.zadaniey.iter().filter(|t| t.done).count();
        let progress = (done as f32 / total as f32) * 100.0;

        // Красивое форматирование вывода
        println!("📋 Список задач (выполнено: {:.1}%):", progress);
        for zadanie in &self.zadaniey {
            let status = if zadanie.done { "✓" } else { " " }; // Галочка для выполненных
            let due_info = match &zadanie.due_date {
                Some(date) => format!(" [срок: {}]", date), // Показываем срок если есть
                None => String::new(),
            };
            println!(
                "{:3} [{}] {}{}", // Формат: "ID [✓] Текст [срок: ...]"
                zadanie.id, status, zadanie.text, due_info
            );
        }
    }

    /// Отметка задачи как выполненной по ID
    fn complete(&mut self, id: usize) -> Result<(), String> {
        match self.zadaniey.iter_mut().find(|t| t.id == id) {
            Some(zadanie) => {
                zadanie.complete();
                Ok(println!("👍 Задача {} выполнена", id))
            }
            None => Err(format!("❌ Задача с ID {} не найдена", id)),
        }
    }

    /// Удаление задачи по ID
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

/// Сохранение задач в JSON файл
fn save_to_file(zadaniey: &[Zadanie]) -> io::Result<()> {
    let json = serde_json::to_string(zadaniey)?; // Сериализация в JSON
    let mut file = OpenOptions::new()
        .write(true)
        .create(true) // Создать если не существует
        .truncate(true) // Очистить если существует
        .open("zadaniey.json")?; // Открыть файл
    file.write_all(json.as_bytes())?; // Записать данные
    Ok(())
}

/// Загрузка задач из JSON файла
fn load_from_file() -> io::Result<Vec<Zadanie>> {
    if !Path::new("zadaniey.json").exists() {
        return Ok(Vec::new()); // Вернуть пустой вектор если файла нет
    }

    let mut file = File::open("zadaniey.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?; // Чтение всего файла
    let zadaniey = serde_json::from_str(&contents)?; // Парсинг JSON
    Ok(zadaniey)
}

fn main() {
    // Загрузка существующих задач или создание нового списка
    let mut todo = match load_from_file() {
        Ok(zadaniey) => {
            // Восстанавливаем next_id как максимальный ID + 1
            let next_id = zadaniey.iter().map(|t| t.id).max().unwrap_or(0) + 1;
            ToDolist { zadaniey, next_id }
        }
        Err(e) => {
            eprintln!("⚠️ Ошибка загрузки: {}. Новый список создан.", e);
            ToDolist::new()
        }
    };

    // Приветствие и текущая дата
    println!("✨ ToDo менеджер v1.0");
    println!("📅 Сегодня: {}", Local::now().format("%Y-%m-%d"));

    // Основной цикл программы
    loop {
        // Вывод меню команд
        println!("\n📌 Команды:");
        println!(" добавить <текст> - Добавить задачу");
        println!(" добавить <текст> до <ГГГГ-ММ-ДД> - Добавить задачу со сроком");
        println!(" список - Показать все задачи");
        println!(" выполнить <ID> - Отметить задачу как выполненную");
        println!(" удалить <ID> - Удалить задачу");
        println!(" выход - Выйти из программы");
        print!("➥ "); // Символ приглашения
        io::Write::flush(&mut io::stdout()).unwrap(); // Сброс буфера вывода

        // Чтение пользовательского ввода
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        // Разбивка ввода на части для анализа команды
        let parts: Vec<&str> = input.splitn(3, ' ').collect();

        // Обработка команд
        match parts[0] {
            // Добавление задачи (с датой или без)
            "добавить" if parts.len() >= 2 => {
                if let Some((_cmd, rest)) = input.split_once(' ') {
                    if let Some((text, date)) = rest.split_once(" до ") {
                        // Формат: "добавить <текст> до <дата>"
                        if let Err(e) = todo.add_with_date(text.to_string(), date) {
                            eprintln!("{}", e);
                        } else {
                            save_to_file(&todo.zadaniey).unwrap();
                        }
                    } else {
                        // Формат: "добавить <текст>"
                        todo.add(rest.to_string());
                        save_to_file(&todo.zadaniey).unwrap();
                    }
                }
            }

            // Вывод списка задач
            "список" => todo.list(),

            // Выполнение задачи по ID
            "выполнить" if parts.len() > 1 => {
                if let Ok(id) = parts[1].parse::<usize>() {
                    if let Err(e) = todo.complete(id) {
                        eprintln!("{}", e);
                    } else {
                        save_to_file(&todo.zadaniey).unwrap();
                    }
                } else {
                    eprintln!("❌ Неверный ID задачи. Введите число.");
                }
            }

            // Удаление задачи по ID
            "удалить" if parts.len() > 1 => {
                if let Ok(id) = parts[1].parse::<usize>() {
                    if let Err(e) = todo.remove(id) {
                        eprintln!("{}", e);
                    } else {
                        save_to_file(&todo.zadaniey).unwrap();
                    }
                } else {
                    eprintln!("❌ Неверный ID задачи. Введите число.");
                }
            }

            // Выход из программы
            "выход" => break,

            // Пустая команда (просто нажатие Enter)
            "" => continue,

            // Неизвестная команда
            _ => {
                if parts[0] == "добавить" {
                    eprintln!("❌ Неверный формат команды. Используйте: добавить <текст задачи> [до <ГГГГ-ММ-ДД>]");
                } else {
                    eprintln!("❌ Неизвестная команда. Доступные команды: добавить, список, выполнить, удалить, выход");
                }
            }
        }
    }
}
