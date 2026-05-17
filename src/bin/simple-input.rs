use eframe::egui;

struct App {
    text: String,
    lines: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            text: String::new(),
            lines: Vec::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Дебаггинг ввода с клавиатуры
        ctx.input(|i| {
            for event in &i.events {
                println!("Событие: {:?}", event);
            }

            if i.key_pressed(egui::Key::Enter) {
                println!("Нажат Enter");
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Простой текстовый редактор");

            // Используем TextEdit с возвращаемым значением для дебаггинга
            let response = ui.text_edit_multiline(&mut self.text);

            // Дебаггинг взаимодействия с полем ввода
            if response.gained_focus() {
                println!("Поле ввода получило фокус");
            }

            if response.lost_focus() {
                println!("Поле ввода потеряло фокус. Текст: '{}'", self.text);
            }

            if response.changed() {
                println!("Текст изменен: '{}'", self.text);
            }

            // Показываем ID поля для дебаггинга
            ui.label(format!("ID поля: {:?}", response.id));

            // Кнопка добавления
            if ui.button("Добавить").clicked() {
                if !self.text.trim().is_empty() {
                    println!("=== ДОБАВЛЕНИЕ ===");
                    println!("Текст: '{}'", self.text);
                    println!("Длина: {} символов", self.text.len());
                    self.lines.push(self.text.clone());
                    self.text.clear();
                } else {
                    println!("Попытка добавить пустой текст");
                }
            }

            // Детальная информация о тексте
            ui.separator();
            ui.heading("Детали текста:");
            ui.label(format!("Текущий текст: '{}'", self.text));
            ui.label(format!("Длина: {} символов", self.text.len()));
            ui.label(format!("Пустой: {}", self.text.is_empty()));
            ui.label(format!("Только пробелы: {}", self.text.trim().is_empty()));

            // Список добавленных текстов с деталями
            ui.heading("Добавленные тексты:");
            for (i, line) in self.lines.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("{}.", i + 1));
                    ui.label(format!("'{}'", line));
                    ui.label(format!("({} символов)", line.len()));
                });
            }
        });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Текстовый редактор",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}
