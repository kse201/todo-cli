use anyhow::anyhow;
use clap::Parser;
use std::path::PathBuf;
use todo_cli::task::*;

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
fn main() -> anyhow::Result<()> {
    let opt = Opt::parse();
    let journal_path = opt
        .journal_fine
        .or_else(find_default_journal_file)
        .ok_or(anyhow!("Failed to find journal file."))?;
    let mut tasks = Tasks::with_journal(journal_path)?;

    match opt.action {
        Action::Add { text } => tasks.add(Task::new(text)),
        Action::List => tasks.list(),
        Action::Done { position } => tasks.complete(position),
    }?;
    Ok(())
}

fn find_default_journal_file() -> Option<PathBuf> {
    home::home_dir().map(|mut path| {
        path.push(".todo-cli.json");
        path
    })
}
