use eframe::egui;
use rusqlite::{Connection, types::Value};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "DBF Viewer GUI",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new()))),
    )
}

struct MyApp {
    tab: usize,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    status: String,
}

impl MyApp {
    fn new() -> Self {
        let conn = Connection::open("sample.db").expect("failed to open sqlite db");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sample (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                value REAL
            );
            INSERT OR IGNORE INTO sample (id, name, value) VALUES
                (1, 'Alpha', 1.5),
                (2, 'Beta', 2.5),
                (3, 'Gamma', 3.5);",
        )
        .expect("failed to init sqlite db");

        let mut stmt = conn.prepare("SELECT * FROM sample").unwrap();
        let ncols = stmt.column_count();
        let columns: Vec<String> = (0..ncols)
            .map(|i| stmt.column_name(i).unwrap().to_owned())
            .collect();
        let rows: Vec<Vec<String>> = stmt
            .query_map([], |row| {
                let values: Vec<String> = (0..ncols)
                    .map(|i| {
                        row.get::<usize, Value>(i)
                            .map(|v| value_to_string(&v))
                            .unwrap_or_default()
                    })
                    .collect();
                Ok(values)
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();

        let row_count = rows.len();

        Self {
            tab: 0,
            columns,
            rows,
            status: format!(
                "sqlite: sample.db, table \"sample\", {} column(s), {} row(s)",
                ncols, row_count
            ),
        }
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::Integer(i) => i.to_string(),
        Value::Real(f) => f.to_string(),
        Value::Text(s) => s.clone(),
        Value::Blob(b) => format!("{:?}", b),
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(self.tab == 0, "Hello").clicked() {
                    self.tab = 0;
                }
                if ui.selectable_label(self.tab == 1, "Table from sqlite").clicked() {
                    self.tab = 1;
                }
            });
            ui.separator();

            match self.tab {
                0 => {
                    ui.label("Hello, world");
                }
                _ => {
                    ui.label(&self.status);
                    ui.separator();
                    egui::ScrollArea::both().show(ui, |ui| {
                        egui::Grid::new("table_grid")
                            .striped(true)
                            .show(ui, |ui| {
                                for col in &self.columns {
                                    ui.strong(col);
                                }
                                ui.end_row();
                                for row in &self.rows {
                                    for cell in row {
                                        ui.label(cell);
                                    }
                                    ui.end_row();
                                }
                            });
                    });
                }
            }
        });
    }
}