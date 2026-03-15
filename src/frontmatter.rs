use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMeta {
    pub id: String,

    /// Creation timestamp (PRD 2.0). Backward compatible with `created_at`.
    #[serde(default, alias = "created_at")]
    pub timestamp: Option<DateTime<Utc>>,

    #[serde(default)]
    pub last_access: Option<DateTime<Utc>>,

    #[serde(default)]
    pub hit_count: u64,

    #[serde(default, with = "hex_u64_opt")]
    pub simhash: Option<u64>,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub links: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FrontmatterDoc {
    pub meta: MemoryMeta,
    pub body: String,
}

pub fn parse(markdown: &str) -> Result<FrontmatterDoc> {
    let mut lines = markdown.lines();
    let first = lines.next().ok_or_else(|| anyhow!("empty document"))?;
    if first.trim() != "---" {
        return Err(anyhow!("missing YAML frontmatter delimiter (`---`)"));
    }

    let mut yaml = String::new();
    for line in &mut lines {
        if line.trim() == "---" {
            break;
        }
        yaml.push_str(line);
        yaml.push('\n');
    }

    let meta: MemoryMeta = serde_yaml::from_str(&yaml).context("parse YAML frontmatter")?;
    let body = lines
        .collect::<Vec<_>>()
        .join("\n")
        .trim_start_matches('\n')
        .to_string();

    Ok(FrontmatterDoc { meta, body })
}

pub fn render(meta: &MemoryMeta, body: &str) -> Result<String> {
    let yaml = serde_yaml::to_string(meta).context("serialize YAML frontmatter")?;
    Ok(format!("---\n{yaml}---\n\n{body}\n"))
}

mod hex_u64_opt {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<u64>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(v) => serializer.serialize_str(&format!("0x{v:016X}")),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        let Some(s) = opt else {
            return Ok(None);
        };
        let s = s.trim();
        let s = s
            .strip_prefix("0x")
            .or_else(|| s.strip_prefix("0X"))
            .unwrap_or(s);
        u64::from_str_radix(s, 16)
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_roundtrip() {
        let now = Utc::now();
        let meta = MemoryMeta {
            id: "mem_test".to_string(),
            timestamp: Some(now),
            last_access: Some(now),
            hit_count: 3,
            simhash: Some(0xFF12A3B4C5D6E789),
            tags: vec!["rust".to_string(), "concurrency".to_string()],
            links: vec!["mem_other".to_string()],
        };
        let md = render(&meta, "# Title\n\nBody").expect("render");
        let doc = parse(&md).expect("parse");
        assert_eq!(doc.meta.id, "mem_test");
        assert!(doc.body.contains("Title"));
        assert_eq!(doc.meta.simhash, meta.simhash);
    }
}
