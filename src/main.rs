use std::fs::OpenOptions;
use std::io::BufReader;

use eframe::{Frame, egui};
use egui::{Color32, RichText, ScrollArea};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ListApp {
    tasks: Vec<Task>,
    #[serde(skip, default = "String::new")]
    new_task_text: String,
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
    Color32::from_rgb(173, 216, 230),
    Color32::from_rgb(144, 238, 144),
    Color32::from_rgb(255, 182, 193),
    Color32::from_rgb(255, 255, 224),
    Color32::from_rgb(216, 191, 216),
    Color32::from_rgb(255, 215, 0),
    Color32::from_rgb(255, 150, 100),
    Color32::from_rgb(100, 200, 255),
    Color32::from_rgb(200, 100, 255),
    Color32::from_rgb(255, 100, 150),
    Color32::from_rgb(100, 255, 150),
    Color32::from_rgb(200, 200, 100),
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
    // Сначала сериализуем в строку для проверки
    match serde_json::to_string_pretty(state) {
        Ok(json_string) => {
            let json_len = json_string.len(); // Сохраняем длину до перемещения
            // Проверяем, что JSON валидный
            if serde_json::from_str::<serde_json::Value>(&json_string).is_ok() {
                match std::fs::write(STATEPATH, &json_string) {
                    Ok(_) => println!("✅ Сохранено успешно ({} байт)", json_len),
                    Err(e) => eprintln!("❌ Ошибка записи файла: {}", e),
                }
            } else {
                eprintln!("❌ Ошибка: создан невалидный JSON");
            }
        }
        Err(e) => eprintln!("❌ Ошибка сериализации: {}", e),
    }
}

fn load_state<T: serde::de::DeserializeOwned>() -> Option<T> {
    let file = OpenOptions::new()
        .read(true)
        .open(STATEPATH)
        .ok()?;
    
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
            });
            self.new_task_text.clear();
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

                for (i, category) in self.categories.iter().enumerate() {
                    let color: Color32 = category.color.into();
                    let button_text = RichText::new(&category.name).color(color);
                    let button = egui::Button::new(button_text).fill(
                        if self.selected_category == i {
                            ctx.style().visuals.widgets.active.bg_fill
                        } else {
                            Color32::TRANSPARENT
                        },
                    );

                    if ui.add(button).clicked() {
                        self.selected_category = i;
                    }
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

                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.new_task_text)
                            .hint_text("Новая задача...")
                            .min_size(egui::Vec2::new(200.0, 0.0)),
                    );
                    if ui.button("Добавить").clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        self.add_task();
                    }
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
                                    ui.checkbox(&mut task.completed, "");

                                    if task.completed {
                                        ui.add(egui::Label::new(
                                            egui::RichText::new(&task.text)
                                                .strikethrough()
                                                .color(Color32::GRAY),
                                        ));
                                    } else {
                                        ui.label(&task.text);
                                    }

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