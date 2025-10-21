use eframe::egui::{self, RichText};
use uuid::Uuid;

#[derive(Default)]
struct Task {
    task_id: Uuid,
    task_text: String,
    done: bool,
}

#[derive(Default)]
struct Model {
    add_task_text_box: String,
    tasks: Vec<Task>,
}

enum Msg {
    TextInput(String),
    Add,
    CheckBox(Uuid, bool),
    Delete(Uuid),
}

fn init() -> Model {
    Model::default()
}

fn update(m: Model, msg: Msg) -> Model {
    match msg {
        Msg::TextInput(task_text) => Model {
            add_task_text_box: task_text,

            ..m
        },

        Msg::Add => {
            let mut tasks = m.tasks;
            tasks.push(Task {
                task_id: Uuid::new_v4(),
                task_text: m.add_task_text_box.clone(),
                done: false,
            });

            Model {
                tasks,
                add_task_text_box: "".to_string(),
            }
        }

        Msg::CheckBox(id, done) => {
            let mut tasks = m.tasks;
            if let Some(task) = tasks.iter_mut().find(|t| t.task_id == id) {
                task.done = done;
            }

            Model { tasks, ..m }
        }

        Msg::Delete(id) => {
            let mut tasks = m.tasks;
            if let Some(task) = tasks.iter().position(|t| t.task_id == id) {
                tasks.remove(task);
            }
            Model { tasks, ..m }
        }
    }
}

fn view(ctx: &egui::Context, m: &Model, tx: &mut Vec<Msg>) {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        let visuals = egui::Visuals::light();
        ctx.set_visuals(visuals);
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Chai Task");
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let mut add_task_text_box = m.add_task_text_box.clone();
                let text_edit_id = ui.make_persistent_id("add_task_text_box");

                let response = ui.add(
                    egui::TextEdit::singleline(&mut add_task_text_box)
                        .hint_text("Add a task...")
                        .id(text_edit_id),
                );

                if response.changed() {
                    tx.push(Msg::TextInput(add_task_text_box));
                }

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    tx.push(Msg::Add);
                    ui.memory_mut(|mem| mem.request_focus(text_edit_id));
                }
            });

            for task in m.tasks.iter().rev() {
                ui.horizontal(|ui| {
                    let mut checked = task.done;

                    let text = if checked {
                        RichText::new(&task.task_text).strikethrough().weak()
                    } else {
                        RichText::new(&task.task_text)
                    };

                    let check_response = ui.checkbox(&mut checked, text);

                    if check_response.changed() {
                        tx.push(Msg::CheckBox(task.task_id, checked));
                    }

                    if checked && ui.button("🗑").clicked() {
                        tx.push(Msg::Delete(task.task_id));
                    }
                });
            }
        });
    });
}

fn main() -> eframe::Result<()> {
    chai_tea::brew("elaichi chai", init, update, view)
}
