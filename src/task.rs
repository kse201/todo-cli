use chrono::serde::ts_seconds;
use chrono::{DateTime, Local, Utc};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom};
use std::path::PathBuf;

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

pub struct Tasks {
    tasks: Vec<Task>,
    file: File,
}

impl Tasks {
    pub fn with_journal(journal_path: PathBuf) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(journal_path)?;
        let tasks = Tasks::collect_task(&file)?;
        Ok(Self { tasks, file })
    }

    pub fn add(&mut self, task: Task) -> Result<()> {
        self.tasks.push(task);
        Ok(())
    }

    pub fn list(&self) -> Result<()> {
        if self.tasks.is_empty() {
            println!("Task list is empty");
        } else {
            let mut order: u32 = 1;
            for task in &self.tasks {
                println!("{}: {}", order, task);
                order += 1;
            }
        }

        Ok(())
    }

    pub fn complete(&mut self, task_position: usize) -> Result<()> {
        if task_position == 0 || task_position > self.tasks.len() {
            return Err(Error::new(ErrorKind::InvalidInput, "Invalid Task ID"));
        }
        self.tasks.remove(task_position - 1);
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
}

impl Drop for Tasks {
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        // Rewind and truncate the file.
        self.file.seek(SeekFrom::Start(0));
        self.file.set_len(0);

        serde_json::to_writer(&mut self.file, &self.tasks);
    }
}
