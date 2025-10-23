use dirs::data_dir;
use eframe::egui::{self, RichText};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

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
    Search,
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
    RescheduleActive(String),
    Edit(Uuid),
    EditInput(Uuid, String),
    EditDone(Uuid),
}

fn init() -> Model {
    let filename = "database.json";
    let mut path = data_dir().expect("no data dir found");
    path.push("cardamom-chai");
    std::fs::create_dir_all(&path).ok();
    path.push(filename);

    let tasks = match std::fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_else(|_| vec![]),
        Err(_) => vec![],
    };

    Model {
        tasks,
        path,
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

            let mut state = TaskState::Normal;
            let mut text = m.add_task_text_box.clone();

            if text.ends_with('?') {
                state = TaskState::Uncertain;
                text = text.trim_end_matches('?').to_string();
            } else if text.ends_with('!') {
                state = TaskState::Chosen;
                text = text.trim_end_matches('!').to_string();
            }

            tasks.push(Task {
                task_id: Uuid::new_v4(),
                task_text: text,
                state,
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

        Msg::RescheduleActive(text) => {
            let mut tasks = m.tasks;
            tasks.push(Task {
                task_id: Uuid::new_v4(),
                task_text: text,
                state: TaskState::Chosen,
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
                if task.done {
                    task.state = TaskState::Normal;
                } else {
                    task.state = match task.state {
                        TaskState::Normal => TaskState::Chosen,
                        TaskState::Chosen => TaskState::Uncertain,
                        TaskState::Uncertain => TaskState::Normal,
                    }
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

        Msg::Edit(id) => {
            let mut edit_tasks = m.edit_tasks;
            edit_tasks.push(id);
            (Model { edit_tasks, ..m }, None)
        }

        Msg::EditInput(id, new_text) => {
            let mut tasks = m.tasks;
            if let Some(task) = tasks.iter_mut().find(|t| t.task_id == id) {
                task.task_text = new_text;
            }

            (Model { tasks, ..m }, None)
        }

        Msg::EditDone(id) => {
            let mut edit_tasks = m.edit_tasks;
            if let Some(edit_task) = edit_tasks.iter().position(|t| *t == id) {
                edit_tasks.remove(edit_task);
            }
            let path = m.path.clone();
            let tasks = m.tasks.clone();

            (Model { edit_tasks, ..m }, Some(Cmd::Write(path, tasks)))
        }
    }
}

fn view(ctx: &egui::Context, m: &Model, tx: &mut Vec<Msg>) {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        let visuals = egui::Visuals::light();
        ctx.set_visuals(visuals);
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (
                egui::TextStyle::Heading,
                egui::FontId::new(24.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::new(15.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::new(14.0, egui::FontFamily::Monospace),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::new(15.0, egui::FontFamily::Proportional),
            ),
            (
                egui::TextStyle::Small,
                egui::FontId::new(10.0, egui::FontFamily::Proportional),
            ),
        ]
        .into();
        ctx.set_style(style);
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        let mut add_task_text_box_has_focus = false;
        let mut task_edit_box_has_focus = false;
        let text_edit_id = ui.make_persistent_id("add_task_text_box");

        ui.heading("cardamom chai");
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let mut add_task_text_box = m.add_task_text_box.clone();

                let response = ui.add(
                    egui::TextEdit::singleline(&mut add_task_text_box)
                        .hint_text("Add a task... '/' to search...")
                        .id(text_edit_id),
                );

                if response.changed() {
                    match m.filter {
                        Filter::Search => {
                            if add_task_text_box.is_empty() {
                                tx.push(Msg::SetFilter(Filter::All));
                            }
                        }
                        _ => {
                            if add_task_text_box.starts_with('/') {
                                tx.push(Msg::SetFilter(Filter::Search));
                            }
                        }
                    }
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
                add_task_text_box_has_focus = response.has_focus();
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                for task in m.tasks.iter().rev().filter(|t| match &m.filter {
                    Filter::All => true,
                    Filter::Active => matches!(t.state, TaskState::Chosen),
                    Filter::Uncertain => matches!(t.state, TaskState::Uncertain),
                    Filter::Search => {
                        let query = m.add_task_text_box.trim_start_matches('/');
                        fuzzy_match(&t.task_text.to_lowercase(), &query.to_lowercase())
                    }
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

                        if m.edit_tasks.contains(&task.task_id) {
                            let mut edit_task_text_box = task.task_text.clone();
                            let _ = ui.checkbox(&mut checked, "");
                            let response =
                                ui.add(egui::TextEdit::singleline(&mut edit_task_text_box));

                            if response.changed() {
                                tx.push(Msg::EditInput(task.task_id, edit_task_text_box));
                            }

                            if response.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                tx.push(Msg::EditDone(task.task_id));
                            }
                            task_edit_box_has_focus |= response.has_focus();
                        } else {
                            let check_response = ui.checkbox(&mut checked, text);

                            if check_response.changed() {
                                tx.push(Msg::CheckBox(task.task_id, checked));
                            }

                            if check_response.double_clicked() {
                                tx.push(Msg::Edit(task.task_id));
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
                            if checked && ui.button("⟲").clicked() {
                                tx.push(Msg::RescheduleActive(task.task_text.clone()));
                                tx.push(Msg::CycleTaskState(task.task_id));
                            }
                        }
                    });
                }
            });
        });
        //hotkeys
        if !add_task_text_box_has_focus && !task_edit_box_has_focus {
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                ui.memory_mut(|mem| mem.request_focus(text_edit_id));
            }

            if ui.input(|i| i.key_pressed(egui::Key::A)) {
                tx.push(Msg::SetFilter(Filter::All));
            }

            if ui.input(|i| i.key_pressed(egui::Key::F)) {
                tx.push(Msg::SetFilter(Filter::Active));
            }

            if ui.input(|i| i.key_pressed(egui::Key::U)) {
                tx.push(Msg::SetFilter(Filter::Uncertain));
            }

            if ui.input(|i| i.key_pressed(egui::Key::D)) {
                tx.push(Msg::SetFilter(Filter::Done));
            }
        }
    });
}

struct SyncState {}

enum Cmd {
    Write(PathBuf, Vec<Task>),
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
        }
    }
}

fn fuzzy_match(haystack: &str, needle: &str) -> bool {
    let mut n_chars = needle.chars();
    let mut current = n_chars.next();
    for c in haystack.chars() {
        if Some(c) == current {
            current = n_chars.next();
            if current.is_none() {
                return true;
            }
        }
    }
    false
}

#[tokio::main]
async fn main() -> eframe::Result<()> {
    chai_tea::brew_async(
        "cardamom-chai",
        init,
        sync_state_init,
        update,
        view,
        run_cmd,
    )
}
