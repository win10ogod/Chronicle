use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use git2::build::CheckoutBuilder;
use git2::{BranchType, Commit, DiffOptions, IndexAddOption, Repository, Sort};

#[derive(Clone, Debug)]
pub struct Git {
    root: PathBuf,
}

impl Git {
    pub fn open(root: &Path) -> Result<Self> {
        let git = Self {
            root: root.to_path_buf(),
        };
        git.ensure_work_tree()?;
        Ok(git)
    }

    pub fn init_if_missing(root: &Path) -> Result<()> {
        if root.join(".git").exists() {
            return Ok(());
        }
        Repository::init(root).context("git init")?;
        Ok(())
    }

    pub fn stage_paths(&mut self, paths: &[PathBuf]) -> Result<()> {
        if paths.is_empty() {
            return Ok(());
        }

        let repo = self.repo()?;
        let mut index = repo.index().context("open git index")?;

        let mut specs = Vec::new();
        for path in paths {
            specs.push(to_repo_pathspec(&self.root, path)?);
        }

        index
            .add_all(specs.iter(), IndexAddOption::DEFAULT, None)
            .context("git index add_all")?;
        index
            .update_all(specs.iter(), None)
            .context("git index update_all")?;
        index.write().context("git index write")?;
        Ok(())
    }

    pub fn commit(&mut self, message: &str) -> Result<()> {
        let repo = self.repo()?;
        let mut index = repo.index().context("open git index")?;
        let tree_id = index.write_tree().context("write tree")?;
        let tree = repo.find_tree(tree_id).context("find tree")?;

        let sig = repo
            .signature()
            .or_else(|_| git2::Signature::now("Chronicle", "chronicle@local"))
            .context("create git signature")?;

        let parent = current_head_commit(&repo).ok();
        if parent.as_ref().is_some_and(|p| p.tree_id() == tree_id) {
            return Ok(());
        }

        match parent {
            Some(ref p) => {
                repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[p])
                    .context("git commit")?;
            }
            None => {
                repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[])
                    .context("git initial commit")?;
            }
        }

        Ok(())
    }

    pub fn checkout_new_branch(&self, name: &str) -> Result<()> {
        let repo = self.repo()?;
        if let Ok(head) = current_head_commit(&repo) {
            repo.branch(name, &head, false)
                .with_context(|| format!("create branch {name}"))?;
        }

        repo.set_head(&format!("refs/heads/{name}"))
            .with_context(|| format!("set HEAD to {name}"))?;
        repo.checkout_head(Some(CheckoutBuilder::new().safe()))
            .context("checkout branch")?;
        Ok(())
    }

    pub fn checkout_branch(&self, name: &str) -> Result<()> {
        let repo = self.repo()?;
        repo.set_head(&format!("refs/heads/{name}"))
            .with_context(|| format!("set HEAD to {name}"))?;
        repo.checkout_head(Some(CheckoutBuilder::new().safe()))
            .context("checkout branch")?;
        Ok(())
    }

    pub fn merge_branch(&self, name: &str) -> Result<()> {
        let repo = self.repo()?;
        let mut head_ref = repo.head().context("get HEAD")?;
        let head_oid = head_ref
            .target()
            .ok_or_else(|| anyhow!("HEAD is unborn (no commits yet)"))?;

        let branch = repo
            .find_branch(name, BranchType::Local)
            .with_context(|| format!("find branch {name}"))?;
        let target_oid = branch
            .get()
            .target()
            .ok_or_else(|| anyhow!("branch {name} has no target"))?;

        if head_oid == target_oid {
            return Ok(());
        }

        if repo
            .graph_descendant_of(head_oid, target_oid)
            .context("check ancestry")?
        {
            // already contains target
            return Ok(());
        }

        if !repo
            .graph_descendant_of(target_oid, head_oid)
            .context("check ancestry")?
        {
            return Err(anyhow!(
                "non-fast-forward merge is not supported (try using Git directly)"
            ));
        }

        head_ref
            .set_target(target_oid, "fast-forward")
            .context("fast-forward")?;
        repo.checkout_head(Some(CheckoutBuilder::new().force()))
            .context("checkout after fast-forward")?;
        Ok(())
    }

    pub fn list_branches(&self) -> Result<Vec<String>> {
        let repo = self.repo()?;
        let mut out = Vec::new();
        let branches = repo.branches(Some(BranchType::Local))?;
        for b in branches {
            let (branch, _) = b?;
            if let Some(name) = branch.name()?.map(|s| s.to_string()) {
                out.push(name);
            }
        }
        out.sort();
        Ok(out)
    }

    pub fn current_branch(&self) -> Result<String> {
        let repo = self.repo()?;
        let head = repo.head().context("get HEAD")?;
        head.shorthand()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("unable to determine current branch"))
    }

    pub fn delete_branch(&self, name: &str, force: bool) -> Result<()> {
        let repo = self.repo()?;
        let mut branch = repo
            .find_branch(name, BranchType::Local)
            .with_context(|| format!("find branch {name}"))?;

        if force {
            // If force, delete regardless of merge status.
            branch.delete().context("delete branch")?;
            return Ok(());
        }

        // `git branch -d` fails if not fully merged. We'll approximate by
        // requiring fast-forward from HEAD.
        let head = current_head_commit(&repo).context("get HEAD commit")?;
        let target_oid = branch
            .get()
            .target()
            .ok_or_else(|| anyhow!("branch {name} has no target"))?;
        if head.id() != target_oid
            && !repo
                .graph_descendant_of(head.id(), target_oid)
                .context("check ancestry")?
        {
            return Err(anyhow!("branch {name} is not fully merged (use --force)"));
        }

        branch.delete().context("delete branch")?;
        Ok(())
    }

    pub fn log_chronicle(&self, limit: Option<usize>) -> Result<String> {
        let repo = self.repo()?;
        let mut revwalk = repo.revwalk().context("create revwalk")?;
        revwalk.push_head().context("revwalk push HEAD")?;
        revwalk.set_sorting(Sort::TIME)?;

        let mut out = String::new();
        let mut n = 0usize;
        for oid in revwalk {
            let oid = oid.context("revwalk oid")?;
            let commit = repo.find_commit(oid).context("find commit")?;
            if !chronicle_changed(&repo, &commit)? {
                continue;
            }

            let short = short_oid(oid);
            let summary = commit.summary().unwrap_or("");
            out.push_str(&format!("{short} {summary}\n"));
            n += 1;
            if limit.is_some_and(|limit| n >= limit) {
                break;
            }
        }
        Ok(out)
    }

    fn ensure_work_tree(&self) -> Result<()> {
        self.repo()
            .map(|_| ())
            .map_err(|_| anyhow!("not inside a Git work tree (run `chronicle init --git-init`)"))
    }

    fn repo(&self) -> Result<Repository> {
        Repository::discover(&self.root).context("open git repository")
    }
}

fn to_repo_pathspec(root: &Path, path: &Path) -> Result<String> {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let s = rel
        .to_string_lossy()
        .replace('\\', "/")
        .trim_matches('/')
        .to_string();
    if s.is_empty() {
        return Err(anyhow!("invalid pathspec for {}", path.display()));
    }
    Ok(s)
}

fn current_head_commit(repo: &Repository) -> Result<Commit<'_>> {
    let head = repo.head().context("get HEAD")?;
    let oid = head
        .target()
        .ok_or_else(|| anyhow!("HEAD is unborn (no commits yet)"))?;
    repo.find_commit(oid).context("find HEAD commit")
}

fn chronicle_changed(repo: &Repository, commit: &Commit<'_>) -> Result<bool> {
    let tree = commit.tree().context("get commit tree")?;
    if commit.parent_count() == 0 {
        return Ok(tree.get_name(".chronicle").is_some());
    }

    let parent = commit.parent(0).context("get parent")?;
    let parent_tree = parent.tree().context("get parent tree")?;
    let mut opts = DiffOptions::new();
    opts.pathspec(".chronicle");
    let diff = repo
        .diff_tree_to_tree(Some(&parent_tree), Some(&tree), Some(&mut opts))
        .context("diff trees")?;
    Ok(diff.deltas().next().is_some())
}

fn short_oid(oid: git2::Oid) -> String {
    let s = oid.to_string();
    s.chars().take(7).collect()
}
