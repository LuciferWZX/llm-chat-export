use std::path::PathBuf;

use anyhow::{Context, Result};
use rusqlite::Connection;

use crate::app::Conversation;

fn db_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .map(|h| h.join("Library/Application Support/Cursor/User/globalStorage/state.vscdb"))
    }
    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|d| d.join("Cursor/User/globalStorage/state.vscdb"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        dirs::config_dir().map(|d| d.join("Cursor/User/globalStorage/state.vscdb"))
    }
}

pub fn fetch_conversations() -> Result<Vec<Conversation>> {
    let db = db_path().context("Cannot determine Cursor data directory")?;
    if !db.exists() {
        anyhow::bail!("Cursor database not found: {:?}", db);
    }

    let conn =
        Connection::open_with_flags(&db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .with_context(|| format!("Failed to open Cursor database: {:?}", db))?;

    let mut stmt = conn
        .prepare("SELECT key, value FROM cursorDiskKV WHERE key LIKE 'composerData:%'")
        .context("Failed to query Cursor database")?;

    let rows = stmt
        .query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        })
        .context("Failed to read Cursor conversations")?;

    let mut convs = Vec::new();

    for row in rows {
        let (_key, value) = match row {
            Ok(r) => r,
            Err(_) => continue,
        };

        let val: serde_json::Value = match serde_json::from_str(&value) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let composer_id = val
            .get("composerId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if composer_id.is_empty() {
            continue;
        }

        // Skip conversations with no messages
        let headers = val
            .get("fullConversationHeadersOnly")
            .and_then(|v| v.as_array());
        if headers.map_or(true, |h| h.is_empty()) {
            continue;
        }

        let created_at_ms = val.get("createdAt").and_then(|v| v.as_i64()).unwrap_or(0);
        let created_at = if created_at_ms > 0 {
            format_epoch_ms(created_at_ms)
        } else {
            String::new()
        };

        // Get first user message text as title
        let text = val
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mode = val
            .get("unifiedMode")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let title = if text.is_empty() {
            format!("[{}] {}", mode, &composer_id[..8.min(composer_id.len())])
        } else {
            let clean: String = text.chars().filter(|c| !c.is_control()).collect();
            let truncated: String = clean.chars().take(80).collect();
            truncated
        };

        convs.push(Conversation {
            id: composer_id,
            title,
            created_at: created_at.clone(),
            last_chat_time: created_at,
            selected: false,
            project: String::new(),
        });
    }

    // Sort by created_at descending
    convs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(convs)
}

pub fn export_conversation(id: &str, title: &str) -> Result<String> {
    let db = db_path().context("Cannot determine Cursor data directory")?;
    let conn =
        Connection::open_with_flags(&db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .with_context(|| format!("Failed to open Cursor database: {:?}", db))?;

    // Get composerData to find message order
    let composer_key = format!("composerData:{}", id);
    let composer_json: String = conn
        .query_row(
            "SELECT value FROM cursorDiskKV WHERE key = ?1",
            [&composer_key],
            |row| row.get(0),
        )
        .with_context(|| format!("Conversation not found: {}", id))?;

    let composer: serde_json::Value =
        serde_json::from_str(&composer_json).context("Failed to parse composer data")?;

    let headers = composer
        .get("fullConversationHeadersOnly")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let mode = composer
        .get("unifiedMode")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let mut md = format!(
        "# {}\n\n**Platform:** Cursor\n**Mode:** {}\n**ID:** {}\n\n---\n\n",
        title, mode, id
    );

    // Fetch each bubble in order
    for header in &headers {
        let bubble_id = match header.get("bubbleId").and_then(|v| v.as_str()) {
            Some(bid) => bid,
            None => continue,
        };
        let bubble_type = header.get("type").and_then(|v| v.as_i64()).unwrap_or(0);

        let bubble_key = format!("bubbleId:{}:{}", id, bubble_id);
        let bubble_json: String = match conn.query_row(
            "SELECT value FROM cursorDiskKV WHERE key = ?1",
            [&bubble_key],
            |row| row.get(0),
        ) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let bubble: serde_json::Value = match serde_json::from_str(&bubble_json) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let text = bubble
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();

        if text.is_empty() {
            continue;
        }

        let role = match bubble_type {
            1 => "User",
            2 => "Assistant",
            _ => "System",
        };

        md.push_str(&format!("## {}\n\n{}\n\n---\n\n", role, text));
    }

    Ok(md)
}

fn format_epoch_ms(ms: i64) -> String {
    let secs = ms / 1000;
    let dt = chrono::DateTime::from_timestamp(secs, 0);
    match dt {
        Some(d) => d.format("%Y-%m-%d %H:%M").to_string(),
        None => String::new(),
    }
}
