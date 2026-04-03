use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::app::Conversation;

const API_BASE: &str = "https://api.manus.im";
const LIST_SESSIONS_PATH: &str = "/session.v1.SessionService/ListSessions";
const GET_SESSION_PATH: &str = "/api/chat/getSessionV2";

fn local_storage_path() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        dirs::home_dir()
            .map(|h| h.join("Library/Application Support/Manus/localStorage.json"))
    }
    #[cfg(target_os = "windows")]
    {
        dirs::config_dir().map(|d| d.join("Manus/localStorage.json"))
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        dirs::config_dir().map(|d| d.join("Manus/localStorage.json"))
    }
}

fn read_token() -> Result<String> {
    let path = local_storage_path().context("Cannot determine Manus data directory")?;
    if !path.exists() {
        anyhow::bail!(
            "Manus localStorage.json not found: {:?}\nPlease make sure Manus desktop app is installed and logged in.",
            path
        );
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read Manus localStorage: {:?}", path))?;
    let val: serde_json::Value =
        serde_json::from_str(&content).context("Failed to parse Manus localStorage.json")?;
    let token = val
        .get("token")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if token.is_empty() {
        anyhow::bail!("No token found in Manus localStorage.json. Please log in to Manus desktop app first.");
    }
    Ok(token)
}

pub fn fetch_conversations() -> Result<Vec<Conversation>> {
    let token = read_token()?;

    let client = reqwest::blocking::Client::new();
    let mut all_sessions = Vec::new();
    let mut offset = 0u64;
    let limit = 50u64;

    loop {
        let body = serde_json::json!({
            "limit": limit,
            "offset": offset,
            "keyword": "",
            "status": []
        });

        let resp = client
            .post(format!("{}{}", API_BASE, LIST_SESSIONS_PATH))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "application/json")
            .header("Connect-Protocol-Version", "1")
            .json(&body)
            .send()
            .context("Failed to call Manus ListSessions API")?;

        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("Manus API returned status {}", status);
        }

        let data: serde_json::Value = resp.json().context("Failed to parse Manus API response")?;

        let sessions = data.get("sessions").and_then(|v| v.as_array());
        let total: u64 = data
            .get("total")
            .and_then(|v| {
                v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .unwrap_or(0);

        if let Some(sessions) = sessions {
            for session in sessions {
                let uid = session
                    .get("uid")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if uid.is_empty() {
                    continue;
                }

                let title = session
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let created_at = session
                    .get("createdAt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let last_message_time = session
                    .get("lastMessageTime")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Format ISO timestamps to YYYY-MM-DD HH:MM
                let created_display = format_iso_timestamp(&created_at);
                let last_display = format_iso_timestamp(&last_message_time);

                let display_title = if title.is_empty() {
                    format!("[Manus] {}", &uid[..8.min(uid.len())])
                } else {
                    title
                };

                all_sessions.push(Conversation {
                    id: uid,
                    title: display_title,
                    created_at: created_display.clone(),
                    last_chat_time: if last_display.is_empty() {
                        created_display
                    } else {
                        last_display
                    },
                    selected: false,
                    project: String::new(),
                });
            }

            offset += sessions.len() as u64;
            if offset >= total || sessions.is_empty() {
                break;
            }
        } else {
            break;
        }
    }

    // Sort by last_chat_time descending
    all_sessions.sort_by(|a, b| b.last_chat_time.cmp(&a.last_chat_time));

    Ok(all_sessions)
}

pub fn export_conversation(id: &str, title: &str) -> Result<String> {
    let token = read_token()?;

    let client = reqwest::blocking::Client::new();

    let mut md = format!(
        "# {}\n\n**Platform:** Manus\n**Session:** {}\n\n---\n\n",
        title, id
    );

    // Fetch the first segment
    let mut next_endpoint: Option<String> = None;
    let mut is_first = true;

    loop {
        let mut url = format!(
            "{}{}?sessionId={}&type=private",
            API_BASE, GET_SESSION_PATH, id
        );
        if is_first {
            url.push_str("&getFirstSegment=true");
            is_first = false;
        }
        if let Some(ref endpoint) = next_endpoint {
            url.push_str(&format!("&startSegmentEndpoint={}", endpoint));
        }

        let resp = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .with_context(|| format!("Failed to fetch Manus session: {}", id))?;

        if !resp.status().is_success() {
            anyhow::bail!("Manus API returned status {} for session {}", resp.status(), id);
        }

        let data: serde_json::Value = resp.json().context("Failed to parse session response")?;
        let session_data = data.get("data").unwrap_or(&data);

        // Process segments
        let segments = session_data
            .get("segments")
            .and_then(|v| v.as_array());

        if let Some(segments) = segments {
            for segment in segments {
                let events = segment.get("events").and_then(|v| v.as_array());
                if let Some(events) = events {
                    for event in events {
                        let event_type = event
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");

                        if event_type == "chat" {
                            let sender = event
                                .get("sender")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");

                            let content = event
                                .get("content")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");

                            if content.is_empty() {
                                continue;
                            }

                            let role = match sender {
                                "user" => "User",
                                "assistant" => "Assistant",
                                _ => sender,
                            };

                            md.push_str(&format!("## {}\n\n{}\n\n---\n\n", role, content));
                        }
                    }
                }
            }
        }

        // Check for more segments via segmentEndpoints
        let segment_endpoints = session_data
            .get("segmentEndpoints")
            .and_then(|v| v.as_array());

        if let Some(endpoints) = segment_endpoints {
            if let Some(last) = endpoints.last() {
                let endpoint_id = last.as_str().unwrap_or("");
                if !endpoint_id.is_empty() && next_endpoint.as_deref() != Some(endpoint_id) {
                    next_endpoint = Some(endpoint_id.to_string());
                    continue;
                }
            }
        }

        break;
    }

    Ok(md)
}

fn format_iso_timestamp(iso: &str) -> String {
    // Input: "2026-04-02T03:09:14.412Z"
    // Output: "2026-04-02 03:09"
    if iso.len() < 16 {
        return iso.to_string();
    }
    iso.get(..16)
        .map(|s| s.replace('T', " "))
        .unwrap_or_default()
}
