use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::git::Git;
use crate::memory::{
    ConsolidateInput, ForgetInput, Layer, MemoryStore, RecallInput, RememberInput,
};
use crate::paths::ProjectPaths;
use crate::wal::{self, CommitMode};

#[derive(Parser, Debug)]
#[command(name = "chronicle")]
#[command(about = "Plain-text, Git-versioned memory store for AI agents", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Project root (defaults to auto-detect from current directory)
    #[arg(long, global = true)]
    root: Option<PathBuf>,

    /// Git commit mode after write/touch operations
    #[arg(long, global = true, value_enum, default_value_t = CommitMode::Async)]
    commit: CommitMode,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Initialize `.chronicle` directory structure in the project
    Init(InitArgs),

    /// Store a new memory block (and commit it to Git)
    Remember(RememberArgs),

    /// Recall memory blocks using the MAMA matching algorithm
    Recall(RecallArgs),

    /// Move eligible short-term memories into long-term storage
    Consolidate(ConsolidateArgs),

    /// Apply forgetting: move low-strength memories to archive or delete
    Forget(ForgetArgs),

    /// Show memory change history (wraps `git log`)
    Log(LogArgs),

    /// Git branch helpers for sandboxed reasoning
    Branch(BranchArgs),

    /// Print shell completion script
    Completions(CompletionsArgs),

    /// Internal: process WAL tasks
    #[command(hide = true)]
    Wal(WalArgs),

    /// Print project status summary
    Status,
}

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Initialize at this path (defaults to current directory / auto-detect)
    path: Option<PathBuf>,

    /// Create the Git repository if missing (`git init`)
    #[arg(long)]
    git_init: bool,
}

#[derive(Args, Debug)]
pub struct RememberArgs {
    /// Memory message/body (Markdown supported)
    #[arg(long, short = 'm')]
    msg: String,

    /// Optional explicit id / filename stem (e.g. `Rust` creates `Rust.md`)
    #[arg(long)]
    id: Option<String>,

    /// Store in short-term or long-term layer
    #[arg(long, value_enum, default_value_t = LayerArg::Short)]
    layer: LayerArg,

    /// Extra tags (repeatable)
    #[arg(long, short = 't')]
    tags: Vec<String>,

    /// Skip Git commit (still writes the file)
    #[arg(long)]
    no_commit: bool,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum LayerArg {
    Short,
    Long,
    Archive,
}

impl From<LayerArg> for Layer {
    fn from(value: LayerArg) -> Self {
        match value {
            LayerArg::Short => Layer::ShortTerm,
            LayerArg::Long => Layer::LongTerm,
            LayerArg::Archive => Layer::Archive,
        }
    }
}

#[derive(Args, Debug)]
pub struct RecallArgs {
    /// Search query
    #[arg(long, short = 'q')]
    query: String,

    /// Number of results to return
    #[arg(
        long = "top-k",
        visible_alias = "top",
        short = 'k',
        default_value_t = 3
    )]
    top_k: usize,

    /// Include archive results
    #[arg(long)]
    include_archive: bool,

    /// Do not update access metadata (hit_count/last_access)
    #[arg(long)]
    no_touch: bool,

    /// Output JSON for agent consumption
    #[arg(long)]
    json: bool,

    /// Skip associative expansion via `[[links]]`
    #[arg(long)]
    no_assoc: bool,

    /// Skip Git commit for access metadata updates
    #[arg(long)]
    no_commit: bool,
}

#[derive(Args, Debug)]
pub struct ConsolidateArgs {
    /// Consolidate if hit_count >= this value
    #[arg(long, default_value_t = 5)]
    min_hits: u64,

    /// Consolidate if age (hours) >= this value (0 disables)
    #[arg(long, default_value_t = 0.0)]
    min_age_hours: f64,

    /// Preview actions without writing
    #[arg(long)]
    dry_run: bool,

    /// Skip Git commit
    #[arg(long)]
    no_commit: bool,
}

#[derive(Args, Debug)]
pub struct ForgetArgs {
    /// Delete a specific memory by id
    #[arg(long, conflicts_with = "threshold")]
    id: Option<String>,

    /// Archive long-term memories whose ACT-R heat is below this threshold (0..1 recommended)
    #[arg(long, conflicts_with = "id")]
    threshold: Option<f64>,

    /// Preview actions without writing
    #[arg(long)]
    dry_run: bool,

    /// Skip Git commit
    #[arg(long)]
    no_commit: bool,
}

#[derive(Args, Debug)]
pub struct LogArgs {
    /// Maximum number of commits to show
    #[arg(long)]
    limit: Option<usize>,
}

#[derive(Args, Debug)]
pub struct BranchArgs {
    #[command(subcommand)]
    command: BranchCommand,
}

#[derive(Subcommand, Debug)]
pub enum BranchCommand {
    /// Create and checkout a new branch
    Create { name: String },
    /// Checkout an existing branch
    Checkout { name: String },
    /// Merge a branch into the current branch
    Merge { name: String },
    /// List local branches
    List,
    /// Print current branch name
    Current,
    /// Delete a local branch
    Delete {
        name: String,
        #[arg(long)]
        force: bool,
    },
}

#[derive(Args, Debug)]
pub struct CompletionsArgs {
    #[arg(value_enum)]
    shell: clap_complete::Shell,
}

#[derive(Args, Debug)]
pub struct WalArgs {
    #[command(subcommand)]
    command: WalCommand,
}

#[derive(Subcommand, Debug)]
pub enum WalCommand {
    Run,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init(args) => {
            let root = args.path.as_deref().or(cli.root.as_deref());
            let paths = ProjectPaths::discover(root)?;
            cmd_init(&paths, args)
        }
        Command::Remember(args) => {
            let paths = ProjectPaths::discover(cli.root.as_deref())?;
            cmd_remember(&paths, args, cli.commit)
        }
        Command::Recall(args) => {
            let paths = ProjectPaths::discover(cli.root.as_deref())?;
            cmd_recall(&paths, args, cli.commit)
        }
        Command::Consolidate(args) => {
            let paths = ProjectPaths::discover(cli.root.as_deref())?;
            cmd_consolidate(&paths, args, cli.commit)
        }
        Command::Forget(args) => {
            let paths = ProjectPaths::discover(cli.root.as_deref())?;
            cmd_forget(&paths, args, cli.commit)
        }
        Command::Log(args) => {
            let paths = ProjectPaths::discover(cli.root.as_deref())?;
            cmd_log(&paths, args)
        }
        Command::Branch(args) => {
            let paths = ProjectPaths::discover(cli.root.as_deref())?;
            cmd_branch(&paths, args)
        }
        Command::Completions(args) => cmd_completions(args),
        Command::Wal(args) => {
            let paths = ProjectPaths::discover(cli.root.as_deref())?;
            cmd_wal(&paths, args)
        }
        Command::Status => {
            let paths = ProjectPaths::discover(cli.root.as_deref())?;
            cmd_status(&paths)
        }
    }
}

fn cmd_init(paths: &ProjectPaths, args: InitArgs) -> Result<()> {
    if args.git_init {
        Git::init_if_missing(paths.root())?;
    }
    MemoryStore::init(paths).context("init chronicle store")?;
    Ok(())
}

fn cmd_remember(paths: &ProjectPaths, args: RememberArgs, commit_mode: CommitMode) -> Result<()> {
    let store = MemoryStore::open(paths)?;

    let input = RememberInput {
        id: args.id,
        layer: args.layer.into(),
        msg: args.msg,
        tags: args.tags,
    };

    let written = store.remember(&input)?;
    if args.no_commit {
        return Ok(());
    }

    commit_after(
        paths,
        commit_mode,
        written.commit_message,
        &written.staged_paths,
    )?;
    Ok(())
}

fn cmd_recall(paths: &ProjectPaths, args: RecallArgs, commit_mode: CommitMode) -> Result<()> {
    let store = MemoryStore::open(paths)?;

    let input = RecallInput {
        query: args.query,
        top: args.top_k,
        include_archive: args.include_archive,
        associative: !args.no_assoc,
        touch: !args.no_touch,
    };

    let result = store.recall(&input)?;

    if args.json {
        let json = serde_json::to_string_pretty(&result.render_json())?;
        println!("{json}");
    } else {
        println!("{}", result.render_markdown());
    }

    if args.no_touch || args.no_commit || result.touched_paths.is_empty() {
        return Ok(());
    }

    commit_after(
        paths,
        commit_mode,
        "Memory access update".to_string(),
        &result.touched_paths,
    )?;
    Ok(())
}

fn cmd_branch(paths: &ProjectPaths, args: BranchArgs) -> Result<()> {
    let git = Git::open(paths.root())?;
    match args.command {
        BranchCommand::Create { name } => git.checkout_new_branch(&name),
        BranchCommand::Checkout { name } => git.checkout_branch(&name),
        BranchCommand::Merge { name } => git.merge_branch(&name),
        BranchCommand::List => {
            for name in git.list_branches()? {
                println!("{name}");
            }
            Ok(())
        }
        BranchCommand::Current => {
            println!("{}", git.current_branch()?);
            Ok(())
        }
        BranchCommand::Delete { name, force } => git.delete_branch(&name, force),
    }
}

fn cmd_consolidate(
    paths: &ProjectPaths,
    args: ConsolidateArgs,
    commit_mode: CommitMode,
) -> Result<()> {
    let store = MemoryStore::open(paths)?;
    let result = store.consolidate(&ConsolidateInput {
        min_hits: args.min_hits,
        min_age_hours: args.min_age_hours,
        dry_run: args.dry_run,
    })?;

    for (from, to) in &result.moved {
        println!("move: {} -> {}", from.display(), to.display());
    }

    if args.dry_run || args.no_commit || result.staged_paths.is_empty() {
        return Ok(());
    }

    commit_after(
        paths,
        commit_mode,
        format!("Memory consolidate ({})", result.moved.len()),
        &result.staged_paths,
    )?;
    Ok(())
}

fn cmd_forget(paths: &ProjectPaths, args: ForgetArgs, commit_mode: CommitMode) -> Result<()> {
    let store = MemoryStore::open(paths)?;
    let input = match (args.id, args.threshold) {
        (Some(id), None) => ForgetInput::ById {
            id,
            dry_run: args.dry_run,
        },
        (None, Some(threshold)) => ForgetInput::ByThreshold {
            threshold,
            dry_run: args.dry_run,
        },
        _ => {
            return Err(anyhow::anyhow!(
                "choose one of: `chronicle forget --id <id>` or `chronicle forget --threshold <val>`"
            ));
        }
    };

    let result = store.forget(&input)?;

    for (from, to) in &result.archived {
        println!("archive: {} -> {}", from.display(), to.display());
    }
    for path in &result.deleted {
        println!("delete: {}", path.display());
    }

    if args.dry_run || args.no_commit || result.staged_paths.is_empty() {
        return Ok(());
    }

    commit_after(
        paths,
        commit_mode,
        format!(
            "Memory forget (archived {}, deleted {})",
            result.archived.len(),
            result.deleted.len()
        ),
        &result.staged_paths,
    )?;
    Ok(())
}

fn cmd_log(paths: &ProjectPaths, args: LogArgs) -> Result<()> {
    let git = Git::open(paths.root())?;
    let out = git.log_chronicle(args.limit)?;
    print!("{out}");
    Ok(())
}

fn cmd_wal(paths: &ProjectPaths, args: WalArgs) -> Result<()> {
    match args.command {
        WalCommand::Run => {
            let processed = wal::run_once(paths)?;
            if processed > 0 {
                eprintln!("chronicle wal: processed {processed} task(s)");
            }
            Ok(())
        }
    }
}

fn commit_after(
    paths: &ProjectPaths,
    mode: CommitMode,
    message: String,
    staged_paths: &[PathBuf],
) -> Result<()> {
    match mode {
        CommitMode::Off => Ok(()),
        CommitMode::Sync => {
            Git::open(paths.root())?;
            wal::enqueue(paths, message, staged_paths)?;
            wal::run_once(paths)?;
            Ok(())
        }
        CommitMode::Async => {
            Git::open(paths.root())?;
            wal::enqueue(paths, message, staged_paths)?;
            spawn_wal_worker(paths)?;
            Ok(())
        }
    }
}

fn spawn_wal_worker(paths: &ProjectPaths) -> Result<()> {
    use std::process::{Command, Stdio};

    let exe = std::env::current_exe().context("locate current executable")?;
    Command::new(exe)
        .arg("--root")
        .arg(paths.root())
        .arg("wal")
        .arg("run")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("spawn wal worker")?;
    Ok(())
}

fn cmd_completions(args: CompletionsArgs) -> Result<()> {
    use clap::CommandFactory;

    let mut cmd = Cli::command();
    clap_complete::generate(args.shell, &mut cmd, "chronicle", &mut std::io::stdout());
    Ok(())
}

fn cmd_status(paths: &ProjectPaths) -> Result<()> {
    let store = MemoryStore::open(paths)?;
    let summary = store.summary()?;
    println!(
        "root: {}\nchronicle: {}\nencryption: {}\nshort_term: {}\nlong_term: {}\narchive: {}",
        paths.root().display(),
        paths.chronicle_dir().display(),
        if store.encryption_enabled() {
            "on"
        } else {
            "off"
        },
        summary.short_term,
        summary.long_term,
        summary.archive,
    );
    Ok(())
}
