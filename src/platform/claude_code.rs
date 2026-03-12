use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::app::Conversation;

fn projects_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("projects"))
}

pub fn fetch_conversations() -> Result<Vec<Conversation>> {
    let projects_dir = projects_dir().context("Cannot find home directory")?;

    if !projects_dir.exists() {
        anyhow::bail!("Claude Code projects directory not found: {:?}", projects_dir);
    }

    let mut convs = Vec::new();

    for project_entry in fs::read_dir(&projects_dir)? {
        let project_entry = project_entry?;
        let project_path = project_entry.path();

        if !project_path.is_dir() {
            continue;
        }

        let project_name = project_entry
            .file_name()
            .to_string_lossy()
            .replace('-', "/")
            .to_string();

        for file_entry in fs::read_dir(&project_path)? {
            let file_entry = file_entry?;
            let file_path = file_entry.path();

            if file_path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                continue;
            }

            let session_id = file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Read first user message as title, created time, and last chat time
            let (title, created, last_chat) = read_session_preview(&file_path);

            convs.push(Conversation {
                id: file_path.to_string_lossy().to_string(),
                title: if title.is_empty() {
                    format!("[{}] {}", &session_id[..8.min(session_id.len())], project_name)
                } else {
                    title
                },
                created_at: created.clone(),
                last_chat_time: if last_chat.is_empty() {
                    created
                } else {
                    last_chat
                },
                selected: false,
                project: project_name.clone(),
            });
        }
    }

    // Sort by timestamp descending (newest first)
    convs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(convs)
}

/// Returns (title, created_at, last_chat_time)
fn read_session_preview(path: &PathBuf) -> (String, String, String) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (String::new(), String::new(), String::new()),
    };

    let mut title = String::new();
    let mut first_timestamp = String::new();
    let mut last_timestamp = String::new();

    for line in content.lines() {
        let val: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let msg_type = val.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if msg_type != "user" && msg_type != "assistant" {
            continue;
        }

        // Track timestamp for every user/assistant message
        if let Some(ts) = val
            .get("timestamp")
            .and_then(|t| t.as_str())
            .and_then(|s| s.get(..16))
        {
            if first_timestamp.is_empty() {
                first_timestamp = ts.to_string();
            }
            last_timestamp = ts.to_string();
        }

        // Extract title from first user message
        if msg_type == "user" && title.is_empty() {
            let msg_content = val
                .pointer("/message/content")
                .and_then(|c| {
                    if let Some(s) = c.as_str() {
                        Some(s.to_string())
                    } else if let Some(arr) = c.as_array() {
                        arr.iter()
                            .filter_map(|item| {
                                if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                                    item.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                                } else {
                                    None
                                }
                            })
                            .next()
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let clean: String = msg_content
                .chars()
                .filter(|c| !c.is_control())
                .collect();
            title = clean.chars().take(80).collect();
        }
    }

    (title, first_timestamp, last_timestamp)
}

pub fn export_conversation(id: &str, title: &str, _project: &str) -> Result<String> {
    // id is the file path for Claude Code
    let content = fs::read_to_string(id)
        .with_context(|| format!("Failed to read session file: {}", id))?;

    let mut md = format!(
        "# {}\n\n**Platform:** Claude Code\n**Session:** {}\n\n---\n\n",
        title,
        std::path::Path::new(id)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(id)
    );

    for line in content.lines() {
        let val: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let msg_type = val.get("type").and_then(|t| t.as_str()).unwrap_or("");

        match msg_type {
            "user" => {
                let text = extract_message_content(&val, "/message/content");
                if !text.is_empty() {
                    md.push_str(&format!("## User\n\n{}\n\n---\n\n", text));
                }
            }
            "assistant" => {
                let text = extract_assistant_content(&val);
                if !text.is_empty() {
                    md.push_str(&format!("## Assistant\n\n{}\n\n---\n\n", text));
                }
            }
            _ => {}
        }
    }

    Ok(md)
}

fn extract_message_content(val: &serde_json::Value, pointer: &str) -> String {
    val.pointer(pointer)
        .and_then(|c| {
            if let Some(s) = c.as_str() {
                Some(s.to_string())
            } else if let Some(arr) = c.as_array() {
                let parts: Vec<String> = arr
                    .iter()
                    .filter_map(|item| {
                        if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                            item.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                if parts.is_empty() {
                    None
                } else {
                    Some(parts.join("\n"))
                }
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn extract_assistant_content(val: &serde_json::Value) -> String {
    let content = val.pointer("/message/content");
    match content {
        Some(c) if c.is_string() => c.as_str().unwrap_or("").to_string(),
        Some(c) if c.is_array() => {
            let arr = c.as_array().unwrap();
            let mut parts = Vec::new();
            for item in arr {
                match item.get("type").and_then(|t| t.as_str()) {
                    Some("text") => {
                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                            parts.push(text.to_string());
                        }
                    }
                    Some("tool_use") => {
                        let name = item
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("tool");
                        parts.push(format!("*[Tool call: {}]*", name));
                    }
                    _ => {}
                }
            }
            parts.join("\n\n")
        }
        _ => String::new(),
    }
}
