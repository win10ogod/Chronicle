use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::crypto::{Crypto, CryptoInit};
use crate::frontmatter::{self, FrontmatterDoc, MemoryMeta};
use crate::lock;
use crate::mama::{self, MamaParams};
use crate::paths::ProjectPaths;
use crate::shadow_index::{self, ShadowEntry, ShadowIndexFile};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer {
    ShortTerm,
    LongTerm,
    Archive,
}

impl Layer {
    fn dir(self, paths: &ProjectPaths) -> PathBuf {
        match self {
            Layer::ShortTerm => paths.short_term_dir(),
            Layer::LongTerm => paths.long_term_dir(),
            Layer::Archive => paths.archive_dir(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStore {
    paths: ProjectPaths,
    crypto: Crypto,
    mama: MamaParams,
    created_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct RememberInput {
    pub id: Option<String>,
    pub layer: Layer,
    pub msg: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RememberOutput {
    pub staged_paths: Vec<PathBuf>,
    pub commit_message: String,
}

#[derive(Debug, Clone)]
pub struct RecallInput {
    pub query: String,
    pub top: usize,
    pub include_archive: bool,
    pub associative: bool,
    pub touch: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecallItem {
    pub id: String,
    pub score: f64,
    pub path: PathBuf,
    pub markdown: String,
}

#[derive(Debug, Clone)]
pub struct RecallResult {
    pub items: Vec<RecallItem>,
    pub touched_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecallJson {
    pub results: Vec<RecallItem>,
}

impl RecallResult {
    pub fn render_markdown(&self) -> String {
        self.items
            .iter()
            .map(|it| it.markdown.trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n\n---\n\n")
    }

    pub fn render_json(&self) -> RecallJson {
        RecallJson {
            results: self.items.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoreSummary {
    pub short_term: usize,
    pub long_term: usize,
    pub archive: usize,
}

#[derive(Debug, Clone)]
pub struct ConsolidateInput {
    pub min_hits: u64,
    pub min_age_hours: f64,
    pub dry_run: bool,
}

#[derive(Debug, Clone)]
pub struct ConsolidateResult {
    pub moved: Vec<(PathBuf, PathBuf)>,
    pub staged_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum ForgetInput {
    /// Delete a specific memory by id.
    ById { id: String, dry_run: bool },

    /// Archive long-term memories whose ACT-R heat is below a threshold.
    ByThreshold { threshold: f64, dry_run: bool },
}

#[derive(Debug, Clone)]
pub struct ForgetResult {
    pub archived: Vec<(PathBuf, PathBuf)>,
    pub deleted: Vec<PathBuf>,
    pub staged_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct StoreConfig {
    #[serde(default)]
    mama: MamaParams,
}

impl MemoryStore {
    pub fn init(paths: &ProjectPaths) -> Result<()> {
        fs::create_dir_all(paths.short_term_dir())
            .with_context(|| format!("create {}", paths.short_term_dir().display()))?;
        fs::create_dir_all(paths.long_term_dir())
            .with_context(|| format!("create {}", paths.long_term_dir().display()))?;
        fs::create_dir_all(paths.archive_dir())
            .with_context(|| format!("create {}", paths.archive_dir().display()))?;

        let cfg_path = paths.chronicle_dir().join("config.yaml");
        if !cfg_path.exists() {
            let cfg = StoreConfig::default();
            let yaml = serde_yaml::to_string(&cfg).context("serialize config")?;
            fs::write(&cfg_path, yaml).with_context(|| format!("write {}", cfg_path.display()))?;
        }

        let readme_path = paths.chronicle_dir().join("README.md");
        if !readme_path.exists() {
            let content = r#"# Chronicle Memory Store

This directory is managed by `chronicle` and is intended to be committed to Git.

- `short_term/`  : working memory (high priority, no forgetting)
- `long_term/`   : consolidated memory (MAMA-scored + forgetting)
- `archive/`     : cold storage for forgotten memory
"#;
            fs::write(&readme_path, content)
                .with_context(|| format!("write {}", readme_path.display()))?;
        }

        let index_path = paths.index_file();
        if !index_path.exists() {
            let idx = ShadowIndexFile {
                version: 2,
                entries: std::collections::BTreeMap::new(),
            };
            idx.write_atomic(&index_path)
                .with_context(|| format!("write {}", index_path.display()))?;
        }

        Ok(())
    }

    pub fn open(paths: &ProjectPaths) -> Result<Self> {
        if !paths.chronicle_dir().is_dir() {
            return Err(anyhow!(
                "missing `.chronicle` directory (run `chronicle init` first)"
            ));
        }

        let cfg_path = paths.chronicle_dir().join("config.yaml");
        let cfg: StoreConfig = match fs::read(&cfg_path) {
            Ok(bytes) => serde_yaml::from_slice(&bytes).context("parse .chronicle/config.yaml")?,
            Err(_) => StoreConfig::default(),
        };

        let CryptoInit {
            crypto,
            created_paths,
        } = Crypto::from_env(&paths.chronicle_dir())?;

        Ok(Self {
            paths: paths.clone(),
            crypto,
            mama: cfg.mama,
            created_paths,
        })
    }

    pub fn encryption_enabled(&self) -> bool {
        self.crypto.is_enabled()
    }

    pub fn remember(&self, input: &RememberInput) -> Result<RememberOutput> {
        lock::with_write_lock(&self.paths.index_lock_file(), || {
            self.remember_locked(input)
        })
    }

    fn remember_locked(&self, input: &RememberInput) -> Result<RememberOutput> {
        let now = Utc::now();
        let id = match &input.id {
            Some(v) => sanitize_id(v),
            None => generate_id(now),
        };

        let mut idx = ShadowIndexFile::load_or_rebuild(&self.paths, &self.crypto)?;
        if idx.entries.contains_key(&id) {
            return Err(anyhow!(
                "memory id already exists in index: {} (choose a different --id)",
                id
            ));
        }

        let file_name = format!("{id}.md");
        let dir = input.layer.dir(&self.paths);
        fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        let path = dir.join(&file_name);
        if path.exists() {
            return Err(anyhow!(
                "memory id already exists: {} (choose a different --id)",
                id
            ));
        }

        let mut tags = normalize_tags(&input.tags);
        tags.extend(normalize_tags(&mama::extract_hashtags(&input.msg)));
        tags.sort();
        tags.dedup();

        let body = ensure_markdown_body(&input.msg, &id);
        let simhash = mama::simhash64_from_text(&body);
        let mut links: Vec<String> = mama::extract_wikilinks(&body)
            .into_iter()
            .map(|s| mama::normalize_link_target(&s))
            .filter(|s| !s.is_empty())
            .collect();
        links.sort();
        links.dedup();

        let meta = MemoryMeta {
            id: id.clone(),
            timestamp: Some(now),
            last_access: Some(now),
            hit_count: 1,
            simhash: Some(simhash),
            tags,
            links,
        };

        let markdown = frontmatter::render(&meta, &body)?;
        let bytes = self.crypto.encrypt_bytes(markdown.as_bytes())?;
        write_atomic(&path, &bytes).with_context(|| format!("write {}", path.display()))?;

        let shadow = ShadowEntry::from_doc(input.layer, &path, &meta, &body, self.paths.root())?;
        idx.entries.insert(id.clone(), shadow);
        idx.write_atomic(&self.paths.index_file())
            .context("write shadow index")?;

        let entity = first_entity_hint(&input.msg, &input.tags);
        let commit_message = match entity {
            Some(e) => format!("Memory update [{e}] detected"),
            None => "Memory update".to_string(),
        };

        let mut staged_paths = vec![path, self.paths.index_file()];
        staged_paths.extend(self.created_paths.iter().cloned());
        Ok(RememberOutput {
            staged_paths,
            commit_message,
        })
    }

    pub fn recall(&self, input: &RecallInput) -> Result<RecallResult> {
        if input.touch {
            lock::with_write_lock(&self.paths.index_lock_file(), || self.recall_locked(input))
        } else {
            lock::with_read_lock(&self.paths.index_lock_file(), || self.recall_locked(input))
        }
    }

    fn recall_locked(&self, input: &RecallInput) -> Result<RecallResult> {
        use std::collections::{HashMap, HashSet, VecDeque};

        let mut idx = ShadowIndexFile::load_or_rebuild(&self.paths, &self.crypto)?;
        if idx.entries.is_empty() {
            return Ok(RecallResult {
                items: Vec::new(),
                touched_paths: Vec::new(),
            });
        }

        let now = Utc::now();
        let query_fp = mama::simhash64_from_text(&input.query);
        let sim_index = idx.build_simhash_index();

        let mut candidate_ids = sim_index.candidates(query_fp, self.mama.candidate_limit);
        if candidate_ids.is_empty() {
            candidate_ids.extend(idx.entries.keys().cloned());
        }
        candidate_ids.retain(|id| {
            idx.entries
                .get(id)
                .is_some_and(|e| input.include_archive || e.layer != Layer::Archive)
        });

        if candidate_ids.is_empty() {
            return Ok(RecallResult {
                items: Vec::new(),
                touched_paths: Vec::new(),
            });
        }

        let id_by_norm: HashMap<String, String> = idx
            .entries
            .keys()
            .map(|id| (id.to_lowercase(), id.clone()))
            .collect();

        let mut base_scores: HashMap<String, f64> = HashMap::new();
        for id in &candidate_ids {
            let Some(entry) = idx.entries.get(id) else {
                continue;
            };
            let score = base_score(&self.mama, &input.query, query_fp, entry, now);
            base_scores.insert(id.clone(), score);
        }

        let mut seeds: Vec<(String, f64)> =
            base_scores.iter().map(|(id, s)| (id.clone(), *s)).collect();
        seeds.sort_by(|a, b| b.1.total_cmp(&a.1));
        seeds.truncate(input.top.max(1));

        let mut assoc_add: HashMap<String, f64> = HashMap::new();
        if input.associative && self.mama.assoc_max_depth > 0 {
            for (seed_id, seed_score) in &seeds {
                if *seed_score <= 0.0 {
                    continue;
                }

                let mut queue: VecDeque<(String, usize)> = VecDeque::new();
                let mut visited: HashSet<String> = HashSet::new();
                visited.insert(seed_id.clone());

                enqueue_neighbors(
                    &mut queue,
                    &mut visited,
                    &idx.entries,
                    &id_by_norm,
                    seed_id,
                    1,
                );

                while let Some((node_id, dist)) = queue.pop_front() {
                    if dist > self.mama.assoc_max_depth {
                        continue;
                    }

                    let add = seed_score * (-self.mama.assoc_lambda * dist as f64).exp();
                    *assoc_add.entry(node_id.clone()).or_insert(0.0) += add;

                    enqueue_neighbors(
                        &mut queue,
                        &mut visited,
                        &idx.entries,
                        &id_by_norm,
                        &node_id,
                        dist + 1,
                    );
                }
            }
        }

        let mut all_ids: HashSet<String> = candidate_ids.into_iter().collect();
        all_ids.extend(assoc_add.keys().cloned());

        for id in all_ids.iter() {
            if base_scores.contains_key(id) {
                continue;
            }
            let Some(entry) = idx.entries.get(id) else {
                continue;
            };
            base_scores.insert(
                id.clone(),
                base_score(&self.mama, &input.query, query_fp, entry, now),
            );
        }

        let mut scored: Vec<(String, f64)> = all_ids
            .into_iter()
            .filter_map(|id| {
                let base = base_scores.get(&id).copied()?;
                let assoc = assoc_add.get(&id).copied().unwrap_or(0.0);
                Some((id, base + assoc))
            })
            .collect();
        scored.sort_by(|a, b| b.1.total_cmp(&a.1));

        let mut items = Vec::new();
        let mut touched_paths = Vec::new();
        let mut touched_any = false;
        for (id, score) in scored.into_iter().take(input.top.max(1)) {
            let Some(entry) = idx.entries.get(&id) else {
                continue;
            };
            let path = entry.abs_path(self.paths.root());
            let loaded = shadow_index::load_doc(&path, &self.crypto)?;

            items.push(RecallItem {
                id: id.clone(),
                score,
                path: path.clone(),
                markdown: loaded.markdown.clone(),
            });

            if input.touch {
                touched_any = true;
                let mut meta = loaded.meta.clone();
                meta.timestamp = meta.timestamp.or(Some(entry.timestamp));
                meta.simhash = Some(entry.simhash);
                meta.links = entry.links.clone();
                meta.tags = entry.tags.clone();
                meta.hit_count = meta.hit_count.saturating_add(1);
                meta.last_access = Some(now);

                let markdown = frontmatter::render(&meta, &loaded.body)?;
                let bytes = self.crypto.encrypt_bytes(markdown.as_bytes())?;
                write_atomic(&path, &bytes)
                    .with_context(|| format!("update {}", path.display()))?;
                touched_paths.push(path.clone());

                if let Some(e) = idx.entries.get_mut(&id) {
                    e.hit_count = meta.hit_count;
                    e.last_access = now;
                    if let Some(ts) = meta.timestamp {
                        e.timestamp = ts;
                    }
                    e.links = meta.links.clone();
                    e.tags = meta.tags.clone();
                    e.simhash = meta.simhash.unwrap_or(e.simhash);
                }
            }
        }

        if touched_any {
            idx.write_atomic(&self.paths.index_file())
                .context("write shadow index")?;
            touched_paths.push(self.paths.index_file());
            touched_paths.extend(self.created_paths.iter().cloned());
        }

        Ok(RecallResult {
            items,
            touched_paths,
        })
    }

    pub fn summary(&self) -> Result<StoreSummary> {
        Ok(StoreSummary {
            short_term: count_md_files(&self.paths.short_term_dir())?,
            long_term: count_md_files(&self.paths.long_term_dir())?,
            archive: count_md_files(&self.paths.archive_dir())?,
        })
    }

    pub fn consolidate(&self, input: &ConsolidateInput) -> Result<ConsolidateResult> {
        lock::with_write_lock(&self.paths.index_lock_file(), || {
            self.consolidate_locked(input)
        })
    }

    fn consolidate_locked(&self, input: &ConsolidateInput) -> Result<ConsolidateResult> {
        let now = Utc::now();
        let mut idx = ShadowIndexFile::load_or_rebuild(&self.paths, &self.crypto)?;
        let docs = self.load_layer(Layer::ShortTerm)?;
        let mut moved = Vec::new();

        for doc in docs {
            let age_anchor = doc.meta.timestamp.or(doc.meta.last_access).unwrap_or(now);
            let age_hours = hours_since(now, age_anchor);
            let by_hits = input.min_hits > 0 && doc.meta.hit_count >= input.min_hits;
            let by_age = input.min_age_hours > 0.0 && age_hours >= input.min_age_hours;
            if !(by_hits || by_age) {
                continue;
            }

            let file_name = doc
                .path
                .file_name()
                .ok_or_else(|| anyhow!("invalid path {}", doc.path.display()))?;
            let dest = self.paths.long_term_dir().join(file_name);
            if dest.exists() {
                return Err(anyhow!(
                    "consolidation target already exists: {}",
                    dest.display()
                ));
            }

            moved.push((doc.path.clone(), dest.clone()));
            if !input.dry_run {
                fs::create_dir_all(self.paths.long_term_dir())
                    .with_context(|| format!("create {}", self.paths.long_term_dir().display()))?;
                fs::rename(&doc.path, &dest).with_context(|| {
                    format!("move {} -> {}", doc.path.display(), dest.display())
                })?;

                if let Some(e) = idx.entries.get_mut(&doc.meta.id) {
                    e.layer = Layer::LongTerm;
                    e.rel_path = dest
                        .strip_prefix(self.paths.root())
                        .map(PathBuf::from)
                        .unwrap_or_else(|_| e.rel_path.clone());
                }
            }
        }

        let mut staged_paths = Vec::new();
        if !input.dry_run && !moved.is_empty() {
            idx.write_atomic(&self.paths.index_file())
                .context("write shadow index")?;
            staged_paths.push(self.paths.short_term_dir());
            staged_paths.push(self.paths.long_term_dir());
            staged_paths.push(self.paths.index_file());
            staged_paths.extend(self.created_paths.iter().cloned());
        }

        Ok(ConsolidateResult {
            moved,
            staged_paths,
        })
    }

    pub fn forget(&self, input: &ForgetInput) -> Result<ForgetResult> {
        lock::with_write_lock(&self.paths.index_lock_file(), || self.forget_locked(input))
    }

    fn forget_locked(&self, input: &ForgetInput) -> Result<ForgetResult> {
        let mut idx = ShadowIndexFile::load_or_rebuild(&self.paths, &self.crypto)?;
        let now = Utc::now();

        match input {
            ForgetInput::ById { id, dry_run } => {
                let archived = Vec::new();
                let mut deleted = Vec::new();

                let target_id = resolve_id_case_insensitive(&idx.entries, id)
                    .ok_or_else(|| anyhow!("memory not found: {id}"))?;
                let Some(entry) = idx.entries.remove(&target_id) else {
                    return Err(anyhow!("memory not found: {id}"));
                };
                let path = entry.abs_path(self.paths.root());
                deleted.push(path.clone());

                if !*dry_run && path.exists() {
                    fs::remove_file(&path).with_context(|| format!("delete {}", path.display()))?;
                }

                if !*dry_run {
                    idx.write_atomic(&self.paths.index_file())
                        .context("write shadow index")?;
                }

                let mut staged_paths = Vec::new();
                if !*dry_run {
                    staged_paths.push(entry.layer.dir(&self.paths));
                    staged_paths.push(self.paths.index_file());
                    staged_paths.extend(self.created_paths.iter().cloned());
                }

                Ok(ForgetResult {
                    archived,
                    deleted,
                    staged_paths,
                })
            }
            ForgetInput::ByThreshold { threshold, dry_run } => {
                let mut archived = Vec::new();
                let deleted = Vec::new();

                let threshold = *threshold;
                for entry in idx.entries.values_mut() {
                    if entry.layer != Layer::LongTerm {
                        continue;
                    }

                    let b = mama::act_r_base_level_activation(
                        now,
                        entry.last_access,
                        entry.hit_count,
                        self.mama.act_alpha,
                        self.mama.act_d,
                    );
                    let heat = mama::sigmoid(b);
                    if heat >= threshold {
                        continue;
                    }

                    let from = entry.abs_path(self.paths.root());
                    let file_name = from
                        .file_name()
                        .ok_or_else(|| anyhow!("invalid path {}", from.display()))?;
                    let to = self.paths.archive_dir().join(file_name);
                    if to.exists() {
                        return Err(anyhow!("archive target already exists: {}", to.display()));
                    }

                    archived.push((from.clone(), to.clone()));
                    if !*dry_run {
                        fs::create_dir_all(self.paths.archive_dir()).with_context(|| {
                            format!("create {}", self.paths.archive_dir().display())
                        })?;
                        fs::rename(&from, &to).with_context(|| {
                            format!("archive {} -> {}", from.display(), to.display())
                        })?;

                        entry.layer = Layer::Archive;
                        entry.rel_path = to
                            .strip_prefix(self.paths.root())
                            .map(PathBuf::from)
                            .unwrap_or_else(|_| entry.rel_path.clone());
                    }
                }

                let mut staged_paths = Vec::new();
                if !*dry_run && !archived.is_empty() {
                    idx.write_atomic(&self.paths.index_file())
                        .context("write shadow index")?;
                    staged_paths.push(self.paths.long_term_dir());
                    staged_paths.push(self.paths.archive_dir());
                    staged_paths.push(self.paths.index_file());
                    staged_paths.extend(self.created_paths.iter().cloned());
                }

                Ok(ForgetResult {
                    archived,
                    deleted,
                    staged_paths,
                })
            }
        }
    }

    fn load_layer(&self, layer: Layer) -> Result<Vec<LoadedDoc>> {
        let dir = layer.dir(&self.paths);
        if !dir.is_dir() {
            return Ok(Vec::new());
        }

        let mut out = Vec::new();
        for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension() != Some(OsStr::new("md")) {
                continue;
            }

            let loaded = self.load_doc(entry.path())?;
            out.push(loaded);
        }
        Ok(out)
    }

    fn load_doc(&self, path: &Path) -> Result<LoadedDoc> {
        let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
        let plaintext = self.crypto.decrypt_bytes(&bytes)?;
        let markdown = String::from_utf8(plaintext).context("memory file is not valid UTF-8")?;

        let FrontmatterDoc { meta, .. } = frontmatter::parse(&markdown)
            .with_context(|| format!("parse frontmatter in {}", path.display()))?;

        Ok(LoadedDoc {
            path: path.to_path_buf(),
            meta,
        })
    }
}

#[derive(Debug, Clone)]
struct LoadedDoc {
    path: PathBuf,
    meta: MemoryMeta,
}

fn count_md_files(dir: &Path) -> Result<usize> {
    if !dir.is_dir() {
        return Ok(0);
    }
    Ok(WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension() == Some(OsStr::new("md")))
        .count())
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;

    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("create temp file in {}", parent.display()))?;
    use std::io::Write as _;
    tmp.write_all(bytes)
        .with_context(|| format!("write temp file for {}", path.display()))?;

    tmp.flush().context("flush temp file")?;
    tmp.as_file().sync_all().context("fsync temp file")?;

    match tmp.persist(path) {
        Ok(_) => Ok(()),
        Err(err) => {
            // On some platforms a direct persist can fail; fall back to rename.
            let (tmp, persist_err) = (err.file, err.error);
            if persist_err.kind() == std::io::ErrorKind::AlreadyExists {
                fs::remove_file(path).with_context(|| format!("remove {}", path.display()))?;
                tmp.persist(path)
                    .map(|_| ())
                    .with_context(|| format!("persist {}", path.display()))
            } else {
                Err(persist_err).with_context(|| format!("persist {}", path.display()))
            }
        }
    }
}

fn generate_id(now: DateTime<Utc>) -> String {
    let ts = now.format("%Y%m%d_%H%M%S").to_string();
    let rand_suffix: u16 = rand::random();
    format!("mem_{ts}_{rand_suffix:04x}")
}

fn sanitize_id(id: &str) -> String {
    let mut out = String::new();
    for ch in id.trim().chars() {
        if ch == '/' || ch == '\\' || ch == '\0' {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    out.trim().to_string()
}

fn ensure_markdown_body(msg: &str, id_hint: &str) -> String {
    let msg_trim = msg.trim();
    if msg_trim.starts_with('#') {
        return msg_trim.to_string();
    }

    let title = msg_trim
        .lines()
        .next()
        .unwrap_or(id_hint)
        .trim()
        .chars()
        .take(48)
        .collect::<String>();
    format!("# {title}\n\n{msg_trim}")
}

fn normalize_tags(tags: &[String]) -> Vec<String> {
    tags.iter()
        .map(|t| t.trim().trim_start_matches('#').to_string())
        .filter(|t| !t.is_empty())
        .collect()
}

fn first_entity_hint(msg: &str, tags: &[String]) -> Option<String> {
    if let Some(link) = mama::extract_wikilinks(msg).into_iter().next() {
        let link = link
            .split('|')
            .next()
            .unwrap_or(&link)
            .split('#')
            .next()
            .unwrap_or(&link)
            .trim()
            .to_string();
        if !link.is_empty() {
            return Some(link);
        }
    }
    if let Some(tag) = mama::extract_hashtags(msg).into_iter().next() {
        return Some(tag);
    }
    tags.iter()
        .map(|t| t.trim().trim_start_matches('#').to_string())
        .find(|t| !t.is_empty())
}

fn hours_since(now: DateTime<Utc>, then: DateTime<Utc>) -> f64 {
    let dt = now.signed_duration_since(then);
    if dt.num_seconds() <= 0 {
        return 0.0;
    }
    (dt.num_seconds() as f64) / 3600.0
}

fn base_score(
    params: &MamaParams,
    query: &str,
    query_fp: u64,
    entry: &ShadowEntry,
    now: DateTime<Utc>,
) -> f64 {
    let s_sem = mama::semantic_similarity(query_fp, entry.simhash);

    let mut entities = Vec::new();
    for link in &entry.links {
        entities.push(mama::Entity {
            kind: mama::EntityKind::WikiLink,
            name: link.clone(),
        });
    }
    for tag in &entry.tags {
        entities.push(mama::Entity {
            kind: mama::EntityKind::HashTag,
            name: tag.clone(),
        });
    }
    let s_ent = mama::entity_score(query, &entities);

    let b = mama::act_r_base_level_activation(
        now,
        entry.last_access,
        entry.hit_count,
        params.act_alpha,
        params.act_d,
    );
    let act = mama::sigmoid(b);

    params.w_sem * s_sem + params.w_ent * s_ent + params.w_act * act
}

fn enqueue_neighbors(
    queue: &mut std::collections::VecDeque<(String, usize)>,
    visited: &mut std::collections::HashSet<String>,
    entries: &std::collections::BTreeMap<String, ShadowEntry>,
    id_by_norm: &std::collections::HashMap<String, String>,
    from_id: &str,
    dist: usize,
) {
    let Some(entry) = entries.get(from_id) else {
        return;
    };

    for raw in &entry.links {
        let norm = mama::normalize_link_target(raw);
        if norm.is_empty() {
            continue;
        }
        let Some(actual) = id_by_norm.get(&norm) else {
            continue;
        };
        if visited.insert(actual.clone()) {
            queue.push_back((actual.clone(), dist));
        }
    }
}

fn resolve_id_case_insensitive(
    entries: &std::collections::BTreeMap<String, ShadowEntry>,
    id: &str,
) -> Option<String> {
    if entries.contains_key(id) {
        return Some(id.to_string());
    }
    let id_norm = id.to_lowercase();
    entries
        .keys()
        .find(|k| k.to_lowercase() == id_norm)
        .cloned()
}
