use anyhow::anyhow;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Local, Utc};
use clap::Parser;
use core::fmt;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub enum Action {
    /// Write tasks to the journal file.
    Add { text: String },

    /// Remove an entry from journal file by position.
    Done { position: usize },

    /// List all tasks in journal file.
    List,
}

#[derive(Debug, Parser)]
#[clap(about, version, author)]
pub struct Opt {
    #[clap(subcommand)]
    pub action: Action,

    /// Use ad differet journal file.
    #[clap(parse(from_os_str), short, long)]
    pub journal_fine: Option<PathBuf>,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub text: String,

    #[serde(with = "ts_seconds")]
    pub created_at: DateTime<Utc>,
}

impl Task {
    pub fn new(text: String) -> Task {
        let created_at = Utc::now();
        Task { text, created_at }
    }
}

impl Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let created_at = self.created_at.with_timezone(&Local).format("%F %H:%M");
        write!(f, "{:<50} [{}]", self.text, created_at)
    }
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    let journal_file = opt
        .journal_fine
        .or_else(find_default_journal_file)
        .ok_or(anyhow!("Failed to find journal file."))?;

    match opt.action {
        Action::Add { text } => add_task(journal_file, Task::new(text)),
        Action::List => list_tasks(journal_file),
        Action::Done { position } => complete_task(journal_file, position),
    }?;
    Ok(())
}

pub fn add_task(journal_path: PathBuf, task: Task) -> Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(journal_path)?;
    let mut tasks = collect_task(&file)?;
    tasks.push(task);
    serde_json::to_writer(file, &tasks)?;

    Ok(())
}

pub fn complete_task(journal_path: PathBuf, task_position: usize) -> Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(journal_path)?;
    let mut tasks = collect_task(&file)?;

    if task_position == 0 || task_position > tasks.len() {
        return Err(Error::new(ErrorKind::InvalidInput, "Invalid Task ID"));
    }
    tasks.remove(task_position - 1);

    file.set_len(0)?;
    serde_json::to_writer(file, &tasks)?;
    Ok(())
}

pub fn list_tasks(journal_path: PathBuf) -> Result<()> {
    let file = OpenOptions::new().read(true).open(journal_path)?;
    let tasks = collect_task(&file)?;

    if tasks.is_empty() {
        println!("Task list is empty");
    } else {
        let mut order: u32 = 1;
        for task in tasks {
            println!("{}: {}", order, task);
            order += 1;
        }
    }

    Ok(())
}

fn collect_task(mut file: &File) -> Result<Vec<Task>> {
    file.seek(SeekFrom::Start(0))?;
    let tasks = match serde_json::from_reader(file) {
        Ok(tasks) => tasks,
        Err(e) if e.is_eof() => Vec::new(),
        Err(e) => Err(e)?,
    };
    file.seek(SeekFrom::Start(0))?;
    Ok(tasks)
}

fn find_default_journal_file() -> Option<PathBuf> {
    home::home_dir().map(|mut path| {
        path.push(".todo-cli.json");
        path
    })
}
