use eframe::egui::{self, RichText};

type Id = usize;

#[derive(Default)]
struct Task {
    task_id: Id,
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
    CheckBox(Id, bool),
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
                task_id: tasks.len(),
                task_text: m.add_task_text_box.clone(),
                done: false,
            });

            Model { tasks, ..m }
        }

        Msg::CheckBox(id, done) => {
            let mut tasks = m.tasks;
            if let Some(task) = tasks.iter_mut().find(|t| t.task_id == id) {
                task.done = done;
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
            ui.label("Add a task: ");
            ui.horizontal(|ui| {
                let mut add_task_text_box = m.add_task_text_box.clone();
                let response = ui.text_edit_singleline(&mut add_task_text_box);
                if response.changed() {
                    tx.push(Msg::TextInput(add_task_text_box));
                }

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    tx.push(Msg::Add);
                }

                if ui.button("add").clicked() {
                    tx.push(Msg::Add);
                }
            });

            for task in m.tasks.iter().rev() {
                let mut checked = task.done;

                let text = if checked {
                    RichText::new(&task.task_text).strikethrough()
                } else {
                    RichText::new(&task.task_text)
                };

                if ui.checkbox(&mut checked, text).changed() {
                    tx.push(Msg::CheckBox(task.task_id, checked));
                }
            }
        });
    });
}

fn main() -> eframe::Result<()> {
    chai_tea::brew("elaichi chai", init, update, view)
}
