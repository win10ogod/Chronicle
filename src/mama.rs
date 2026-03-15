use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use xxhash_rust::xxh3::Xxh3;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MamaParams {
    /// Weight for semantic similarity (SimHash + Hamming).
    #[serde(default = "default_w_sem")]
    pub w_sem: f64,

    /// Weight for entity / tag boost.
    #[serde(default = "default_w_ent")]
    pub w_ent: f64,

    /// Weight for ACT-R base-level activation.
    #[serde(default = "default_w_act")]
    pub w_act: f64,

    /// ACT-R decay factor `d` (default 0.5).
    #[serde(default = "default_act_d")]
    pub act_d: f64,

    /// ACT-R scale `alpha`.
    #[serde(default = "default_act_alpha")]
    pub act_alpha: f64,

    /// Spreading activation damping `lambda`.
    #[serde(default = "default_assoc_lambda")]
    pub assoc_lambda: f64,

    /// Max BFS depth for spreading activation.
    #[serde(default = "default_assoc_max_depth")]
    pub assoc_max_depth: usize,

    /// Candidate cap from SimHash index before full scoring.
    #[serde(default = "default_candidate_limit")]
    pub candidate_limit: usize,
}

impl Default for MamaParams {
    fn default() -> Self {
        Self {
            w_sem: default_w_sem(),
            w_ent: default_w_ent(),
            w_act: default_w_act(),
            act_d: default_act_d(),
            act_alpha: default_act_alpha(),
            assoc_lambda: default_assoc_lambda(),
            assoc_max_depth: default_assoc_max_depth(),
            candidate_limit: default_candidate_limit(),
        }
    }
}

fn default_w_sem() -> f64 {
    1.0
}
fn default_w_ent() -> f64 {
    0.15
}
fn default_w_act() -> f64 {
    0.35
}
fn default_act_d() -> f64 {
    0.5
}
fn default_act_alpha() -> f64 {
    1.0
}
fn default_assoc_lambda() -> f64 {
    0.8
}
fn default_assoc_max_depth() -> usize {
    2
}
fn default_candidate_limit() -> usize {
    512
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityKind {
    WikiLink,
    HashTag,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Entity {
    pub kind: EntityKind,
    pub name: String,
}

pub fn extract_wikilinks(text: &str) -> Vec<String> {
    static WIKI_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = WIKI_RE.get_or_init(|| Regex::new(r"\[\[([^\[\]]+)\]\]").expect("valid regex"));
    re.captures_iter(text)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn extract_hashtags(text: &str) -> Vec<String> {
    static TAG_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let re = TAG_RE.get_or_init(|| Regex::new(r"#([\p{L}\p{N}_-]+)").expect("valid regex"));
    re.captures_iter(text)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().trim().to_string()))
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn entity_score(query: &str, entities: &[Entity]) -> f64 {
    let query_norm = query.to_lowercase();
    let mut seen = HashSet::new();
    let mut score = 0.0;
    for e in entities {
        let name_norm = e.name.to_lowercase();
        if !seen.insert((e.kind, name_norm.clone())) {
            continue;
        }
        if query_norm.contains(&name_norm) {
            score += match e.kind {
                EntityKind::WikiLink => 1.0,
                EntityKind::HashTag => 0.5,
            };
        }
    }
    score
}

pub fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();

    for ch in text.chars() {
        if is_cjk(ch) {
            if !cur.is_empty() {
                out.push(normalize_token(&cur));
                cur.clear();
            }
            out.push(ch.to_string());
            continue;
        }

        if ch.is_alphanumeric() || ch == '_' {
            cur.push(ch);
            continue;
        }

        if !cur.is_empty() {
            out.push(normalize_token(&cur));
            cur.clear();
        }
    }

    if !cur.is_empty() {
        out.push(normalize_token(&cur));
    }

    out.into_iter().filter(|s| !s.is_empty()).collect()
}

fn normalize_token(s: &str) -> String {
    s.to_lowercase()
}

fn is_cjk(ch: char) -> bool {
    matches!(
        ch as u32,
        0x3040..=0x309F | // Hiragana
        0x30A0..=0x30FF | // Katakana
        0x3400..=0x4DBF | // CJK Unified Ideographs Extension A
        0x4E00..=0x9FFF | // CJK Unified Ideographs
        0xF900..=0xFAFF | // CJK Compatibility Ideographs
        0xAC00..=0xD7AF | // Hangul Syllables
        0x20000..=0x2A6DF | // CJK Unified Ideographs Extension B
        0x2A700..=0x2B73F | // Extension C
        0x2B740..=0x2B81F | // Extension D
        0x2B820..=0x2CEAF | // Extension E
        0x2CEB0..=0x2EBEF // Extension F
    )
}

pub fn simhash64_from_text(text: &str) -> u64 {
    simhash64_from_tokens(&tokenize(text))
}

pub fn simhash64_from_tokens(tokens: &[String]) -> u64 {
    if tokens.is_empty() {
        return 0;
    }

    let mut tf: HashMap<&str, i64> = HashMap::new();
    for tok in tokens {
        if tok.is_empty() {
            continue;
        }
        *tf.entry(tok.as_str()).or_insert(0) += 1;
    }

    let mut acc = [0i64; 64];
    for (term, weight) in tf {
        let hash = xxh3_64(term.as_bytes());
        for (bit, slot) in acc.iter_mut().enumerate() {
            let mask = 1u64 << bit;
            if (hash & mask) != 0 {
                *slot += weight;
            } else {
                *slot -= weight;
            }
        }
    }

    let mut out = 0u64;
    for (bit, slot) in acc.iter().enumerate() {
        if *slot > 0 {
            out |= 1u64 << bit;
        }
    }
    out
}

fn xxh3_64(bytes: &[u8]) -> u64 {
    let mut h = Xxh3::new();
    use std::hash::Hasher as _;
    h.write(bytes);
    h.finish()
}

pub fn hamming_distance(a: u64, b: u64) -> u32 {
    (a ^ b).count_ones()
}

pub fn semantic_similarity(query_fp: u64, mem_fp: u64) -> f64 {
    1.0 - (hamming_distance(query_fp, mem_fp) as f64 / 64.0)
}

pub fn act_r_base_level_activation(
    now: DateTime<Utc>,
    last_access: DateTime<Utc>,
    hit_count: u64,
    act_alpha: f64,
    decay_d: f64,
) -> f64 {
    let dt = now.signed_duration_since(last_access);
    let dt_hours = duration_hours_clamped(dt);

    act_alpha * (1.0 + hit_count as f64).ln() - decay_d * (1.0 + dt_hours).ln()
}

pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

fn duration_hours_clamped(dt: Duration) -> f64 {
    let secs = dt.num_seconds();
    if secs <= 0 {
        return 0.0;
    }
    (secs as f64) / 3600.0
}

pub fn normalize_link_target(link: &str) -> String {
    let link = link.trim();
    if link.is_empty() {
        return String::new();
    }

    // Obsidian-style alias `[[target|alias]]` and heading `[[target#heading]]`.
    let base = link
        .split('|')
        .next()
        .unwrap_or(link)
        .split('#')
        .next()
        .unwrap_or(link)
        .trim();

    let base = base
        .rsplit('/')
        .next()
        .unwrap_or(base)
        .trim_end_matches(".md")
        .trim();

    base.to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_splits_ascii_and_cjk() {
        let toks = tokenize("Rust 併發鎖 error42");
        assert!(toks.contains(&"rust".to_string()));
        assert!(toks.contains(&"併".to_string()));
        assert!(toks.contains(&"發".to_string()));
        assert!(toks.contains(&"鎖".to_string()));
        assert!(toks.contains(&"error42".to_string()));
    }

    #[test]
    fn simhash_is_similar_for_small_edits() {
        let a = simhash64_from_text("Rust file lock with fd-lock");
        let b = simhash64_from_text("Rust file locking with fd-lock");
        let sim = semantic_similarity(a, b);
        assert!(sim > 0.6, "expected similarity > 0.6, got {sim}");
    }

    #[test]
    fn normalize_link_target_handles_alias_and_heading() {
        assert_eq!(normalize_link_target("Rust"), "rust");
        assert_eq!(normalize_link_target("Rust.md"), "rust");
        assert_eq!(normalize_link_target("Rust|語言"), "rust");
        assert_eq!(normalize_link_target("Rust#Locks"), "rust");
        assert_eq!(normalize_link_target("dir/Rust"), "rust");
    }

    #[test]
    fn act_r_decays_with_time() {
        let now = Utc::now();
        let recent = now - Duration::hours(1);
        let old = now - Duration::hours(1000);
        let b_recent = act_r_base_level_activation(now, recent, 3, 1.0, 0.5);
        let b_old = act_r_base_level_activation(now, old, 3, 1.0, 0.5);
        assert!(b_recent > b_old);
    }
}
