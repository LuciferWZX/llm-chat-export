use anyhow::{Context, Result};
use serde::Deserialize;

use crate::app::Conversation;

const BASE_URL: &str = "https://chatgpt.com/backend-api";

#[derive(Deserialize)]
struct ConvListResponse {
    items: Vec<ConvItem>,
}

#[derive(Deserialize)]
struct ConvItem {
    id: String,
    title: Option<String>,
    create_time: Option<f64>,
    update_time: Option<f64>,
}

pub fn fetch_conversations(token: &str) -> Result<Vec<Conversation>> {
    let client = reqwest::blocking::Client::new();
    let mut all = Vec::new();
    let mut offset = 0;
    let limit = 100;

    loop {
        let url = format!(
            "{}/conversations?offset={}&limit={}&order=updated",
            BASE_URL, offset, limit
        );
        let resp: ConvListResponse = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .context("ChatGPT API request failed")?
            .error_for_status()
            .context("ChatGPT API returned error status")?
            .json()
            .context("Failed to parse ChatGPT response")?;

        if resp.items.is_empty() {
            break;
        }

        for item in &resp.items {
            all.push(Conversation {
                id: item.id.clone(),
                title: item.title.clone().unwrap_or_else(|| "Untitled".into()),
                created_at: format_ts(item.create_time.unwrap_or(0.0)),
                last_chat_time: format_ts(
                    item.update_time
                        .unwrap_or(item.create_time.unwrap_or(0.0)),
                ),
                selected: false,
                project: String::new(),
            });
        }

        if resp.items.len() < limit {
            break;
        }
        offset += limit;
    }

    Ok(all)
}

pub fn export_conversation(token: &str, id: &str, title: &str) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let url = format!("{}/conversation/{}", BASE_URL, id);
    let resp: serde_json::Value = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .context("ChatGPT conversation detail request failed")?
        .error_for_status()
        .context("ChatGPT API returned error status")?
        .json()
        .context("Failed to parse conversation detail")?;

    let conv_title = resp
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or(title);

    let mut md = format!("# {}\n\n**Platform:** ChatGPT\n**ID:** {}\n\n---\n\n", conv_title, id);

    if let Some(mapping) = resp.get("mapping").and_then(|m| m.as_object()) {
        let messages = traverse_mapping(mapping);
        for (role, content) in messages {
            let label = match role.as_str() {
                "user" => "User",
                "assistant" => "Assistant",
                _ => &role,
            };
            md.push_str(&format!("## {}\n\n{}\n\n---\n\n", label, content));
        }
    }

    Ok(md)
}

fn traverse_mapping(
    mapping: &serde_json::Map<String, serde_json::Value>,
) -> Vec<(String, String)> {
    // Find root node: parent is null or parent not in mapping
    let mut root_id = None;
    for (id, node) in mapping {
        let parent = node.get("parent").and_then(|p| p.as_str());
        match parent {
            None => {
                root_id = Some(id.clone());
                break;
            }
            Some(pid) if !mapping.contains_key(pid) => {
                root_id = Some(id.clone());
                break;
            }
            _ => {}
        }
    }

    let mut messages = Vec::new();
    let mut current = root_id;

    while let Some(id) = current {
        if let Some(node) = mapping.get(&id) {
            if let Some(msg) = node.get("message") {
                let role = msg
                    .pointer("/author/role")
                    .and_then(|r| r.as_str())
                    .unwrap_or("");

                let content = msg
                    .pointer("/content/parts")
                    .and_then(|p| p.as_array())
                    .map(|parts| {
                        parts
                            .iter()
                            .filter_map(|p| p.as_str())
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .unwrap_or_default();

                if !content.is_empty() && (role == "user" || role == "assistant") {
                    messages.push((role.to_string(), content));
                }
            }

            current = node
                .get("children")
                .and_then(|c| c.as_array())
                .and_then(|c| c.first())
                .and_then(|c| c.as_str())
                .map(|s| s.to_string());
        } else {
            break;
        }
    }

    messages
}

fn format_ts(ts: f64) -> String {
    if ts == 0.0 {
        return String::new();
    }
    let secs = ts as i64;
    let nsecs = ((ts - secs as f64) * 1_000_000_000.0).max(0.0) as u32;
    chrono::DateTime::from_timestamp(secs, nsecs)
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_default()
}
