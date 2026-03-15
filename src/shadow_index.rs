use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::crypto::Crypto;
use crate::frontmatter::{self, FrontmatterDoc, MemoryMeta};
use crate::mama;
use crate::memory::Layer;
use crate::paths::ProjectPaths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowIndexFile {
    pub version: u32,

    #[serde(default)]
    pub entries: BTreeMap<String, ShadowEntry>,
}

impl ShadowIndexFile {
    pub fn load_or_rebuild(paths: &ProjectPaths, crypto: &Crypto) -> Result<Self> {
        let path = paths.index_file();
        let existing = fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<ShadowIndexFile>(&bytes).ok());

        if let Some(idx) = existing
            && idx.version == 2
        {
            if idx.entries.is_empty() && chronicle_md_exists(paths)? {
                return Self::rebuild(paths, crypto);
            }
            return Ok(idx);
        }

        Self::rebuild(paths, crypto)
    }

    pub fn rebuild(paths: &ProjectPaths, crypto: &Crypto) -> Result<Self> {
        let mut entries = BTreeMap::new();
        for layer in [Layer::ShortTerm, Layer::LongTerm, Layer::Archive] {
            let dir = match layer {
                Layer::ShortTerm => paths.short_term_dir(),
                Layer::LongTerm => paths.long_term_dir(),
                Layer::Archive => paths.archive_dir(),
            };

            if !dir.is_dir() {
                continue;
            }

            for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
                if !entry.file_type().is_file() {
                    continue;
                }
                if entry.path().extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }

                let loaded = load_doc(entry.path(), crypto)?;
                let shadow = ShadowEntry::from_doc(
                    layer,
                    entry.path(),
                    &loaded.meta,
                    &loaded.body,
                    paths.root(),
                )?;
                entries.insert(shadow.id.clone(), shadow);
            }
        }

        Ok(Self {
            version: 2,
            entries,
        })
    }

    pub fn write_atomic(&self, path: &Path) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(self).context("serialize index")?;
        atomic_write(path, &bytes)
    }

    pub fn build_simhash_index(&self) -> SimHashIndex {
        SimHashIndex::build(&self.entries)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowEntry {
    pub id: String,
    pub layer: Layer,
    pub timestamp: DateTime<Utc>,
    pub last_access: DateTime<Utc>,
    pub hit_count: u64,
    pub simhash: u64,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub links: Vec<String>,

    /// Path relative to project root.
    pub rel_path: PathBuf,
}

impl ShadowEntry {
    pub fn from_doc(
        layer: Layer,
        abs_path: &Path,
        meta: &MemoryMeta,
        body: &str,
        root: &Path,
    ) -> Result<Self> {
        let ts = meta
            .timestamp
            .or(meta.last_access)
            .ok_or_else(|| anyhow!("missing timestamp/last_access for {}", meta.id))?;
        let last = meta
            .last_access
            .or(meta.timestamp)
            .ok_or_else(|| anyhow!("missing last_access/timestamp for {}", meta.id))?;

        let simhash = meta
            .simhash
            .unwrap_or_else(|| mama::simhash64_from_text(body));

        let mut links = meta.links.clone();
        if links.is_empty() {
            links = mama::extract_wikilinks(body)
                .into_iter()
                .map(|s| mama::normalize_link_target(&s))
                .filter(|s| !s.is_empty())
                .collect();
            links.sort();
            links.dedup();
        }

        let rel_path = abs_path
            .strip_prefix(root)
            .map(PathBuf::from)
            .unwrap_or_else(|_| guess_rel_path(root, layer, &meta.id));

        Ok(Self {
            id: meta.id.clone(),
            layer,
            timestamp: ts,
            last_access: last,
            hit_count: meta.hit_count,
            simhash,
            tags: meta.tags.clone(),
            links,
            rel_path,
        })
    }

    pub fn abs_path(&self, root: &Path) -> PathBuf {
        root.join(&self.rel_path)
    }
}

#[derive(Debug, Clone)]
pub struct LoadedDoc {
    pub meta: MemoryMeta,
    pub body: String,
    pub markdown: String,
}

pub fn load_doc(path: &Path, crypto: &Crypto) -> Result<LoadedDoc> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let plaintext = crypto.decrypt_bytes(&bytes)?;
    let markdown = String::from_utf8(plaintext).context("memory file is not valid UTF-8")?;
    let FrontmatterDoc { meta, body } =
        frontmatter::parse(&markdown).with_context(|| format!("parse {}", path.display()))?;
    Ok(LoadedDoc {
        meta,
        body,
        markdown,
    })
}

fn guess_rel_path(root: &Path, layer: Layer, id: &str) -> PathBuf {
    let file = format!("{id}.md");
    let rel = PathBuf::from(".chronicle").join(match layer {
        Layer::ShortTerm => "short_term",
        Layer::LongTerm => "long_term",
        Layer::Archive => "archive",
    });
    let rel = rel.join(&file);
    let abs = root.join(&rel);
    if abs.exists() {
        return rel;
    }
    // Fallback to just the file name under `.chronicle` to avoid panics.
    PathBuf::from(".chronicle").join(file)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
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

#[derive(Debug, Clone)]
pub struct SimHashIndex {
    blocks: [HashMap<u16, Vec<String>>; 4],
}

impl SimHashIndex {
    pub fn build(entries: &BTreeMap<String, ShadowEntry>) -> Self {
        let mut blocks: [HashMap<u16, Vec<String>>; 4] = [
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
        ];

        for (id, entry) in entries {
            let keys = block_keys(entry.simhash);
            for (i, key) in keys.into_iter().enumerate() {
                blocks[i].entry(key).or_default().push(id.clone());
            }
        }

        Self { blocks }
    }

    pub fn candidates(&self, query_fp: u64, limit: usize) -> Vec<String> {
        let keys = block_keys(query_fp);
        let mut seen = HashSet::new();
        let mut out = Vec::new();

        for (i, key) in keys.into_iter().enumerate() {
            let Some(list) = self.blocks[i].get(&key) else {
                continue;
            };
            for id in list {
                if out.len() >= limit {
                    return out;
                }
                if seen.insert(id.clone()) {
                    out.push(id.clone());
                }
            }
        }

        out
    }
}

fn block_keys(fp: u64) -> [u16; 4] {
    [
        (fp & 0xFFFF) as u16,
        ((fp >> 16) & 0xFFFF) as u16,
        ((fp >> 32) & 0xFFFF) as u16,
        ((fp >> 48) & 0xFFFF) as u16,
    ]
}

fn chronicle_md_exists(paths: &ProjectPaths) -> Result<bool> {
    for dir in [
        paths.short_term_dir(),
        paths.long_term_dir(),
        paths.archive_dir(),
    ] {
        if !dir.is_dir() {
            continue;
        }
        if WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| {
                e.file_type().is_file()
                    && e.path().extension().and_then(|s| s.to_str()) == Some("md")
            })
        {
            return Ok(true);
        }
    }
    Ok(false)
}
