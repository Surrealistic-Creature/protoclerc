use chrono::{DateTime, Local, Utc};
use eframe::{Frame, egui};
use egui::{Color32, RichText, ScrollArea};
use serde::{Deserialize, Serialize};
use std::io::BufReader;

#[derive(Serialize, Deserialize)]
struct ListApp {
    tasks: Vec<Task>,
    #[serde(skip, default = "String::new")]
    new_task_text: String,
    #[serde(skip, default = "String::new")]
    task_description: String,
    categories: Vec<Category>,
    #[serde(skip, default = "String::new")]
    new_category_name: String,
    selected_category: usize,
    next_color_index: usize,
}

impl Default for ListApp {
    fn default() -> Self {
        Self {
            tasks: Vec::new(),
            new_task_text: String::new(),
            task_description: String::new(),
            categories: Vec::new(),
            new_category_name: String::new(),
            selected_category: 0,
            next_color_index: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Task {
    text: String,
    completed: bool,
    category: usize,
    task_description: String,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Category {
    name: String,
    color: SerializableColor,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
struct SerializableColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl From<Color32> for SerializableColor {
    fn from(color: Color32) -> Self {
        Self {
            r: color.r(),
            g: color.g(),
            b: color.b(),
            a: color.a(),
        }
    }
}

impl From<SerializableColor> for Color32 {
    fn from(color: SerializableColor) -> Self {
        Color32::from_rgba_premultiplied(color.r, color.g, color.b, color.a)
    }
}

// Палитра цветов для категорий
const CATEGORY_COLORS: [Color32; 12] = [
    Color32::from_rgb(173, 216, 230), // light blue
    Color32::from_rgb(144, 238, 144), // light green
    Color32::from_rgb(255, 182, 193), // light pink
    Color32::from_rgb(255, 255, 224), // light yellow
    Color32::from_rgb(216, 191, 216), // light purple
    Color32::from_rgb(255, 215, 0),   // gold
    Color32::from_rgb(255, 150, 100), // orange
    Color32::from_rgb(100, 200, 255), // light blue 2
    Color32::from_rgb(200, 100, 255), // purple
    Color32::from_rgb(255, 100, 150), // pink
    Color32::from_rgb(100, 255, 150), // mint
    Color32::from_rgb(200, 200, 100), // olive
];

impl Default for Category {
    fn default() -> Self {
        Self {
            name: "Общие".to_string(),
            color: SerializableColor::from(Color32::GRAY),
        }
    }
}

const STATEPATH: &str = "state.json";

fn save_state<T: Serialize>(state: &T) {
    match serde_json::to_string_pretty(state) {
        Ok(json_string) => match std::fs::write(STATEPATH, &json_string) {
            Ok(_) => println!("✅ Сохранено успешно ({} байт)", json_string.len()),
            Err(e) => eprintln!("❌ Ошибка записи файла: {}", e),
        },
        Err(e) => eprintln!("❌ Ошибка сериализации: {}", e),
    }
}

fn load_state<T: serde::de::DeserializeOwned>() -> Option<T> {
    let file = std::fs::File::open(STATEPATH).ok()?;
    let reader = BufReader::new(file);

    match serde_json::from_reader(reader) {
        Ok(state) => {
            println!("✅ Загружено успешно");
            Some(state)
        }
        Err(e) => {
            eprintln!("❌ Ошибка загрузки: {}", e);
            None
        }
    }
}

impl ListApp {
    fn add_task(&mut self) {
        if !self.new_task_text.trim().is_empty() {
            self.tasks.push(Task {
                text: self.new_task_text.clone(),
                completed: false,
                category: self.selected_category,
                task_description: self.task_description.clone(),
                created_at: Utc::now(),
                completed_at: None,
            });
            self.new_task_text.clear();
            self.task_description.clear();
        }
    }

    fn add_category(&mut self) {
        if !self.new_category_name.trim().is_empty() {
            let color = CATEGORY_COLORS[self.next_color_index % CATEGORY_COLORS.len()];
            self.next_color_index += 1;

            self.categories.push(Category {
                name: self.new_category_name.clone(),
                color: SerializableColor::from(color),
            });
            self.new_category_name.clear();
        }
    }

    fn delete_category(&mut self, index: usize) -> bool {
        // Проверка валидности индекса
        if index >= self.categories.len() {
            return false;
        }

        // Нельзя удалить первые три категории (Общие, Работа, Личное)
        if index < 3 {
            return false;
        }

        // Перенос задач удаляемой категории в "Общие" (индекс 0)
        for task in self.tasks.iter_mut() {
            if task.category == index {
                task.category = 0;
            } else if task.category > index {
                task.category -= 1;
            }
        }

        // Удаление категории
        self.categories.remove(index);

        // Корректировка выбранной категории
        if self.selected_category == index {
            self.selected_category = 0;
        } else if self.selected_category > index {
            self.selected_category -= 1;
        }

        true
    }

    fn delete_completed_tasks(&mut self) {
        self.tasks.retain(|task| !task.completed);
    }
}

impl eframe::App for ListApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        if self.categories.is_empty() {
            self.categories.push(Category {
                name: "Общие".to_string(),
                color: SerializableColor::from(CATEGORY_COLORS[0]),
            });
            self.categories.push(Category {
                name: "Работа".to_string(),
                color: SerializableColor::from(CATEGORY_COLORS[1]),
            });
            self.categories.push(Category {
                name: "Личное".to_string(),
                color: SerializableColor::from(CATEGORY_COLORS[2]),
            });
            self.next_color_index = 3;
        }

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Менеджер задач с категориями");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("💾 Сохранить").clicked() {
                        save_state(self);
                    }
                    if ui.button("📂 Загрузить").clicked() {
                        if let Some(loaded) = load_state::<ListApp>() {
                            *self = loaded;
                        }
                    }
                });
            });
            ui.separator();
        });

        egui::SidePanel::left("categories")
            .min_width(150.0)
            .show(ctx, |ui| {
                ui.heading("Категории");

                // Временные переменные для сбора изменений
                let mut new_selected = self.selected_category;
                let mut category_to_delete: Option<usize> = None;

                for (i, category) in self.categories.iter().enumerate() {
                    let color: Color32 = category.color.into();
                    let button_text = RichText::new(&category.name).color(color);
                    let button =
                        egui::Button::new(button_text).fill(if self.selected_category == i {
                            ctx.style().visuals.widgets.active.bg_fill
                        } else {
                            Color32::TRANSPARENT
                        });

                    ui.horizontal(|ui| {
                        if ui.add(button).clicked() {
                            new_selected = i;
                        }

                        // Кнопка удаления категории (только для индекса >= 3)
                        if i >= 3 {
                            if ui.button("🗑️").clicked() {
                                category_to_delete = Some(i);
                            }
                        }
                    });
                }

                // Применяем изменения после цикла
                self.selected_category = new_selected;
                if let Some(index) = category_to_delete {
                    self.delete_category(index);
                }

                ui.separator();

                ui.heading("Добавить категорию");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_category_name)
                            .hint_text("Новая категория")
                            .min_size(egui::Vec2::new(120.0, 0.0)),
                    );
                    if ui.button("➕").clicked() {
                        self.add_category();
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.selected_category < self.categories.len() {
                let current_category = &self.categories[self.selected_category];

                ui.heading(format!("Задачи: {}", current_category.name));
                ui.separator();

                let total_tasks = self
                    .tasks
                    .iter()
                    .filter(|t| t.category == self.selected_category)
                    .count();
                let completed_tasks = self
                    .tasks
                    .iter()
                    .filter(|t| t.category == self.selected_category && t.completed)
                    .count();

                ui.horizontal(|ui| {
                    ui.label(format!("Всего: {}", total_tasks));
                    ui.label(format!("Выполнено: {}", completed_tasks));
                    if total_tasks > 0 {
                        ui.label(format!(
                            "Прогресс: {:.0}%",
                            (completed_tasks as f32 / total_tasks as f32) * 100.0
                        ));
                    }
                });

                ui.separator();

                // Секция добавления задачи
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_task_text)
                                .hint_text("Название задачи...")
                                .min_size(egui::Vec2::new(200.0, 0.0)),
                        );
                        if ui.button("Добавить").clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            self.add_task();
                        }
                    });

                    ui.add(
                        egui::TextEdit::multiline(&mut self.task_description)
                            .hint_text("Описание задачи (опционально)...")
                            .min_size(egui::Vec2::new(200.0, 50.0)),
                    );
                });

                ui.separator();

                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .max_height(400.0)
                    .show(ui, |ui| {
                        let mut tasks_to_remove = Vec::new();

                        for (i, task) in self.tasks.iter_mut().enumerate() {
                            if task.category == self.selected_category {
                                ui.horizontal(|ui| {
                                    // Запоминаем старый статус до чекбокса
                                    let was_completed = task.completed;

                                    // Чекбокс (пользователь может изменить статус)
                                    ui.checkbox(&mut task.completed, "");

                                    // Проверяем, изменился ли статус
                                    if !was_completed && task.completed {
                                        // Только что отметили как выполненную
                                        task.completed_at = Some(Utc::now());
                                    } else if was_completed && !task.completed {
                                        // Только что сняли отметку
                                        task.completed_at = None;
                                    }

                                    ui.vertical(|ui| {
                                        if task.completed {
                                            ui.add(egui::Label::new(
                                                egui::RichText::new(&task.text)
                                                    .strikethrough()
                                                    .color(Color32::GRAY),
                                            ));
                                        } else {
                                            ui.label(&task.text);
                                        }

                                        // Показываем время создания (конвертированное в локальное)
                                        let local_created = task.created_at.with_timezone(&Local);
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "Создано: {}",
                                                local_created.format("%d.%m.%Y %H:%M")
                                            ))
                                            .size(10.0)
                                            .color(Color32::GRAY),
                                        );

                                        // Показываем время выполнения, если задача выполнена (конвертированное в локальное)
                                        if let Some(completed_time) = task.completed_at {
                                            let local_completed =
                                                completed_time.with_timezone(&Local);
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "Выполнено: {}",
                                                    local_completed.format("%d.%m.%Y %H:%M")
                                                ))
                                                .size(10.0)
                                                .color(Color32::GREEN),
                                            );
                                        }

                                        // Показываем описание, если оно есть
                                        if !task.task_description.is_empty() {
                                            ui.label(
                                                egui::RichText::new(&task.task_description)
                                                    .size(12.0)
                                                    .color(Color32::GRAY)
                                                    .italics(),
                                            );
                                        }
                                    });

                                    if ui.button("❌").clicked() {
                                        tasks_to_remove.push(i);
                                    }
                                });
                            }
                        }

                        for &index in tasks_to_remove.iter().rev() {
                            self.tasks.remove(index);
                        }
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("💾 Сохранить").clicked() {
                        save_state(self);
                    }
                    if ui.button("📂 Загрузить").clicked() {
                        if let Some(loaded) = load_state::<ListApp>() {
                            *self = loaded;
                        }
                    }
                });

                if ui.button("🗑️ Удалить выполненные").clicked() {
                    self.delete_completed_tasks();
                }

                // Обновляем заголовок окна
                let current_tasks_count = self
                    .tasks
                    .iter()
                    .filter(|t| t.category == self.selected_category && !t.completed)
                    .count();

                ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                    format!(
                        "Задачи: {} - {} невыполненных",
                        self.categories[self.selected_category].name, current_tasks_count
                    )
                    .into(),
                ));
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Менеджер задач с категориями",
        options,
        Box::new(|_cc| {
            // Загружаем сохраненное состояние при старте
            let app = load_state::<ListApp>().unwrap_or_default();
            Ok(Box::new(app))
        }),
    )
}
