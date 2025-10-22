//use async_trait::async_trait;
use eframe::egui::{self, RichText};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/*
#[async_trait]
pub trait Storage: Send + Sync {
    async fn add(&self, task: Task) -> Result<(), Box<dyn std::error::Error>>;
    async fn update(&self, task: Task) -> Result<(), Box<dyn std::error::Error>>;
    async fn delete(&self, task: Task) -> Result<(), Box<dyn std::error::Error>>;
    async fn list(&self, task: Task) -> Result<(), Box<dyn std::error::Error>>;
    async fn get(&self, task: Task) -> Result<(), Box<dyn std::error::Error>>;
}
*/

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Task {
    task_id: Uuid,
    task_text: String,
    done: bool,
}

#[derive(PartialEq, Default, Copy, Clone)]
enum Filter {
    #[default]
    All,
    Active,
    Done,
}

#[derive(Default)]
struct Model {
    path: PathBuf,
    add_task_text_box: String,
    tasks: Vec<Task>,
    filter: Filter,
    chosen_tasks: Vec<Uuid>,
}

enum Msg {
    TextInput(String),
    Choose(Uuid),
    Unchoose(Uuid),
    Add,
    CheckBox(Uuid, bool),
    Delete(Uuid),
    SetFilter(Filter),
    //TasksLoaded(Vec<Task>),
}

fn init() -> Model {
    let path = "database.json";
    let tasks = match std::fs::read_to_string(path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_else(|_| vec![]),
        Err(_) => vec![],
    };

    Model {
        tasks,
        path: path.into(),
        ..Default::default()
    }
}

fn update(m: Model, msg: Msg) -> (Model, Option<Cmd>) {
    match msg {
        Msg::TextInput(task_text) => (
            Model {
                add_task_text_box: task_text,

                ..m
            },
            None,
        ),

        Msg::Add => {
            let mut tasks = m.tasks;
            tasks.push(Task {
                task_id: Uuid::new_v4(),
                task_text: m.add_task_text_box.clone(),
                done: false,
            });

            (
                Model {
                    tasks: tasks.clone(),
                    add_task_text_box: "".to_string(),
                    path: m.path.clone(),
                    ..m
                },
                Some(Cmd::Write(m.path.clone(), tasks)),
            )
        }

        Msg::CheckBox(id, done) => {
            let mut tasks = m.tasks;
            if let Some(task) = tasks.iter_mut().find(|t| t.task_id == id) {
                task.done = done;
            }

            (
                Model {
                    tasks: tasks.clone(),
                    path: m.path.clone(),
                    ..m
                },
                Some(Cmd::Write(m.path.clone(), tasks)),
            )
        }

        Msg::Delete(id) => {
            let mut tasks = m.tasks;
            if let Some(task) = tasks.iter().position(|t| t.task_id == id) {
                tasks.remove(task);
            }
            (
                Model {
                    tasks: tasks.clone(),
                    path: m.path.clone(),
                    ..m
                },
                Some(Cmd::Write(m.path.clone(), tasks)),
            )
        }
        Msg::SetFilter(filter) => (Model { filter, ..m }, None),
        Msg::Choose(id) => {
            let mut chosen_tasks = m.chosen_tasks;
            chosen_tasks.push(id);
            (Model { chosen_tasks, ..m }, None)
        }

        Msg::Unchoose(id) => {
            let mut chosen_tasks = m.chosen_tasks;
            if let Some(chosen_task) = chosen_tasks.iter().position(|t| *t == id) {
                chosen_tasks.remove(chosen_task);
            }

            (Model { chosen_tasks, ..m }, None)
        } //Msg::TasksLoaded(_) => (m, None),
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

                let mut filter = m.filter;
                let mut changed = false;

                changed |= ui
                    .selectable_value(&mut filter, Filter::All, "All")
                    .changed();
                changed |= ui
                    .selectable_value(&mut filter, Filter::Active, "Active")
                    .changed();
                changed |= ui
                    .selectable_value(&mut filter, Filter::Done, "Done")
                    .changed();

                if changed {
                    tx.push(Msg::SetFilter(filter));
                }
                //if ui.button("Hidden").clicked() { tx.push(Msg::SetFilter(Filter::All)); }
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                for task in m.tasks.iter().rev().filter(|t| match &m.filter {
                    Filter::All => true,
                    Filter::Active => m.chosen_tasks.contains(&t.task_id),
                    Filter::Done => t.done,
                }) {
                    ui.horizontal(|ui| {
                        let mut checked = task.done;

                        let text = if checked {
                            RichText::new(&task.task_text).strikethrough().weak()
                        } else if m.chosen_tasks.contains(&task.task_id) {
                            RichText::new(&task.task_text)
                                .color(egui::Color32::from_rgb(32, 159, 181))
                                .underline()
                        } else {
                            RichText::new(&task.task_text)
                        };

                        let check_response = ui.checkbox(&mut checked, text);

                        if check_response.changed() {
                            tx.push(Msg::CheckBox(task.task_id, checked));
                        }

                        if check_response.secondary_clicked() {
                            if m.chosen_tasks.contains(&task.task_id) {
                                tx.push(Msg::Unchoose(task.task_id));
                            } else {
                                tx.push(Msg::Choose(task.task_id));
                            }
                        }

                        if checked && ui.button("🗑").clicked() {
                            tx.push(Msg::Delete(task.task_id));
                        }
                    });
                }
            });
        });
    });
}

struct SyncState {}

enum Cmd {
    Write(PathBuf, Vec<Task>),
    //Read(PathBuf),
}

fn sync_state_init() -> SyncState {
    SyncState {}
}

fn run_cmd(cmd: Cmd, _sync_state: &mut SyncState, _tx: chai_tea::ChaiSender<Msg>) {
    match cmd {
        Cmd::Write(path, tasks) => {
            tokio::spawn(async move {
                let json = serde_json::to_string_pretty(&tasks).expect("failed to serialize");
                tokio::fs::write(path, json).await.ok();
            });
        } //_ => {}
    }
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    chai_tea::brew_async("elaichi chai", init, sync_state_init, update, view, run_cmd)
}
