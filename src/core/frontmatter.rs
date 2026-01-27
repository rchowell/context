use crate::core::document::Document;
use crate::error::Result;
use serde_yaml::{self, Value};
use std::collections::HashMap;
use std::path::PathBuf;

/// Parse frontmatter and body from document content
///
/// Expects YAML frontmatter between `---` delimiters at the start of the file.
/// Returns Document with frontmatter parsed and body content
pub fn parse(path: PathBuf, content: &str) -> Result<Document> {
    let (frontmatter_str, body) = extract_frontmatter(content).ok_or_else(|| {
        crate::error::ContextError::InvalidDocument("No YAML frontmatter found".to_string())
    })?;

    let frontmatter: Value = serde_yaml::from_str(&frontmatter_str)?;
    let fm = frontmatter.as_mapping().ok_or_else(|| {
        crate::error::ContextError::InvalidDocument("Invalid frontmatter format".to_string())
    })?;

    let slug = fm
        .get(Value::String("slug".to_string()))
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            crate::error::ContextError::InvalidDocument(
                "Missing or invalid 'slug' field".to_string(),
            )
        })?
        .to_string();

    let description = fm
        .get(Value::String("description".to_string()))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let references =
        if let Some(Value::Mapping(refs_map)) = fm.get(Value::String("references".to_string())) {
            let mut refs = HashMap::new();
            for (key, val) in refs_map {
                if let (Some(k), Some(v)) = (key.as_str(), val.as_str()) {
                    refs.insert(k.to_string(), v.to_string());
                }
            }
            refs
        } else {
            HashMap::new()
        };

    let updated = fm
        .get(Value::String("updated".to_string()))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(Document::new(
        path,
        slug,
        description,
        references,
        updated,
        body,
    ))
}

/// Serialize Document back to complete file content with YAML frontmatter
pub fn serialize(document: &Document) -> Result<String> {
    let mut fm_map = serde_yaml::Mapping::new();

    fm_map.insert(
        Value::String("slug".to_string()),
        Value::String(document.slug.clone()),
    );

    fm_map.insert(
        Value::String("description".to_string()),
        Value::String(document.description.clone()),
    );

    let mut refs_map = serde_yaml::Mapping::new();
    for (path, hash) in &document.references {
        refs_map.insert(Value::String(path.clone()), Value::String(hash.clone()));
    }
    fm_map.insert(
        Value::String("references".to_string()),
        Value::Mapping(refs_map),
    );

    fm_map.insert(
        Value::String("updated".to_string()),
        Value::String(document.updated.clone()),
    );

    let frontmatter = serde_yaml::to_string(&fm_map)?;
    Ok(format!("---\n{}---\n\n{}", frontmatter, document.body))
}

/// Extract YAML frontmatter from content
/// Returns (frontmatter_str, body) or None if no frontmatter found
fn extract_frontmatter(content: &str) -> Option<(String, String)> {
    if !content.starts_with("---\n") {
        return None;
    }

    let content_after_first = &content[4..];
    let end_delimiter_pos = content_after_first.find("\n---\n")?;

    let frontmatter = &content_after_first[..end_delimiter_pos];
    let body_start = end_delimiter_pos + 5; // length of "\n---\n"
    let body = if body_start < content_after_first.len() {
        &content_after_first[body_start..]
    } else {
        ""
    };

    Some((frontmatter.to_string(), body.trim_start().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_frontmatter() {
        let content = "---\nslug: test\n---\n\nBody content";
        let (fm, body) = extract_frontmatter(content).unwrap();
        assert_eq!(fm, "slug: test");
        assert_eq!(body, "Body content");
    }

    #[test]
    fn test_parse_document() {
        let content = r"---
slug: auth
description: Authentication system
references:
  src/auth/mod.rs: 8a3b2c1
  src/auth/jwt.rs: f4e5d6a
updated: 2025-01-21
---

# Authentication

This is the body.
";
        let doc = parse(PathBuf::from("test.md"), content).unwrap();
        assert_eq!(doc.slug, "auth");
        assert_eq!(doc.description, "Authentication system");
        assert_eq!(
            doc.references.get("src/auth/mod.rs"),
            Some(&"8a3b2c1".to_string())
        );
        assert!(doc.body.contains("# Authentication"));
    }
}
