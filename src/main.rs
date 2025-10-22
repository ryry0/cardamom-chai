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
    #[serde(default)]
    state: TaskState,
}

#[derive(PartialEq, Default, Copy, Clone, Serialize, Deserialize)]
enum TaskState {
    #[default]
    Normal,
    Chosen,
    Uncertain,
}

#[derive(PartialEq, Default, Copy, Clone)]
enum Filter {
    #[default]
    All,
    Active,
    Uncertain,
    Done,
}

#[derive(Default)]
struct Model {
    path: PathBuf,
    add_task_text_box: String,
    tasks: Vec<Task>,
    filter: Filter,
    edit_tasks: Vec<Uuid>,
}

enum Msg {
    TextInput(String),
    Add,
    CheckBox(Uuid, bool),
    Delete(Uuid),
    SetFilter(Filter),
    CycleTaskState(Uuid),
    Reschedule(String),
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
                ..Default::default()
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

        Msg::Reschedule(text) => {
            let mut tasks = m.tasks;
            tasks.push(Task {
                task_id: Uuid::new_v4(),
                task_text: text,
                ..Default::default()
            });

            (
                Model {
                    tasks: tasks.clone(),
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

        Msg::CycleTaskState(id) => {
            let mut tasks = m.tasks;
            if let Some(task) = tasks.iter_mut().find(|t| t.task_id == id) {
                task.state = match task.state {
                    TaskState::Normal => TaskState::Chosen,
                    TaskState::Chosen => TaskState::Uncertain,
                    TaskState::Uncertain => TaskState::Normal,
                }
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
        /*
        Msg::Edit(id) => {
            let mut edit_tasks = m.edit_tasks;
            edit_tasks.push(id);
            (Model { chosen_tasks, ..m }, None)
        }

        Msg::Unchoose(id) => {
            let mut edit_tasks = m.edit_tasks;
            if let Some(edit_task) = edit_tasks.iter().position(|t| *t == id) {
                edit_tasks.remove(edit_task);
            }

            (Model { edit_tasks, ..m }, None)
        } //Msg::TasksLoaded(_) => (m, None),
        */
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
                    .selectable_value(&mut filter, Filter::Uncertain, "Uncertain")
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
                    Filter::Active => matches!(t.state, TaskState::Chosen),
                    Filter::Uncertain => matches!(t.state, TaskState::Uncertain),
                    Filter::Done => t.done,
                }) {
                    ui.horizontal(|ui| {
                        let mut checked = task.done;

                        let text = if checked {
                            RichText::new(&task.task_text).strikethrough().weak()
                        } else {
                            match task.state {
                                TaskState::Normal => RichText::new(&task.task_text),
                                TaskState::Chosen => RichText::new(&task.task_text)
                                    .color(egui::Color32::from_rgb(32, 159, 181))
                                    .underline(),
                                TaskState::Uncertain => {
                                    RichText::new(format!("{}?", &task.task_text))
                                        .color(egui::Color32::from_rgb(234, 118, 203))
                                }
                            }
                        };

                        let check_response = ui.checkbox(&mut checked, text);

                        if check_response.changed() {
                            tx.push(Msg::CheckBox(task.task_id, checked));
                        }

                        if check_response.secondary_clicked() {
                            tx.push(Msg::CycleTaskState(task.task_id));
                        }

                        if (checked || matches!(task.state, TaskState::Uncertain))
                            && ui.button("🗑").clicked()
                        {
                            tx.push(Msg::Delete(task.task_id));
                        }
                        if checked && ui.button("🔁").clicked() {
                            tx.push(Msg::Reschedule(task.task_text.clone()));
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
