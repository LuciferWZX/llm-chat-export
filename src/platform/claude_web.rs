use anyhow::{Context, Result};
use serde::Deserialize;

use crate::app::Conversation;

const BASE_URL: &str = "https://claude.ai/api";

#[derive(Deserialize)]
struct Organization {
    uuid: String,
}

#[derive(Deserialize)]
struct ConvItem {
    uuid: String,
    name: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

#[derive(Deserialize)]
struct ConvDetail {
    name: Option<String>,
    chat_messages: Option<Vec<ChatMessage>>,
}

#[derive(Deserialize)]
struct ChatMessage {
    sender: Option<String>,
    text: Option<String>,
    content: Option<Vec<ContentBlock>>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: Option<String>,
    text: Option<String>,
}

fn build_client(session_key: &str) -> reqwest::blocking::Client {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::COOKIE,
        reqwest::header::HeaderValue::from_str(&format!("sessionKey={}", session_key))
            .unwrap_or_else(|_| reqwest::header::HeaderValue::from_static("")),
    );
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("Mozilla/5.0"),
    );
    reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}

fn get_org_id(client: &reqwest::blocking::Client) -> Result<String> {
    let orgs: Vec<Organization> = client
        .get(format!("{}/organizations", BASE_URL))
        .send()
        .context("Failed to fetch Claude organizations")?
        .error_for_status()
        .context("Claude API returned error (check session key)")?
        .json()
        .context("Failed to parse organizations response")?;

    orgs.into_iter()
        .next()
        .map(|o| o.uuid)
        .ok_or_else(|| anyhow::anyhow!("No organization found"))
}

pub fn fetch_conversations(session_key: &str) -> Result<Vec<Conversation>> {
    let client = build_client(session_key);
    let org_id = get_org_id(&client)?;

    let items: Vec<ConvItem> = client
        .get(format!(
            "{}/organizations/{}/chat_conversations",
            BASE_URL, org_id
        ))
        .send()
        .context("Failed to fetch Claude conversations")?
        .error_for_status()
        .context("Claude API returned error")?
        .json()
        .context("Failed to parse conversations list")?;

    let convs = items
        .into_iter()
        .map(|item| {
            let created = item
                .created_at
                .as_deref()
                .and_then(|s| s.get(..16))
                .unwrap_or("")
                .to_string();
            let updated = item
                .updated_at
                .as_deref()
                .and_then(|s| s.get(..16))
                .unwrap_or("")
                .to_string();
            Conversation {
                id: item.uuid,
                title: item.name.unwrap_or_else(|| "Untitled".into()),
                created_at: created.clone(),
                last_chat_time: if updated.is_empty() {
                    created
                } else {
                    updated
                },
                selected: false,
                project: org_id.clone(),
            }
        })
        .collect();

    Ok(convs)
}

pub fn export_conversation(session_key: &str, id: &str, title: &str) -> Result<String> {
    let client = build_client(session_key);

    // org_id is stored in project field
    let org_id = {
        let org = get_org_id(&client)?;
        org
    };

    let url = format!(
        "{}/organizations/{}/chat_conversations/{}?tree=True&rendering_mode=messages",
        BASE_URL, org_id, id
    );

    let detail: ConvDetail = client
        .get(&url)
        .send()
        .context("Failed to fetch conversation detail")?
        .error_for_status()
        .context("Claude API returned error")?
        .json()
        .context("Failed to parse conversation detail")?;

    let conv_title = detail.name.as_deref().unwrap_or(title);
    let mut md = format!(
        "# {}\n\n**Platform:** Claude\n**ID:** {}\n\n---\n\n",
        conv_title, id
    );

    if let Some(messages) = detail.chat_messages {
        for msg in messages {
            let role = msg.sender.as_deref().unwrap_or("unknown");
            let label = match role {
                "human" => "User",
                "assistant" => "Assistant",
                _ => role,
            };

            let text = if let Some(text) = &msg.text {
                text.clone()
            } else if let Some(blocks) = &msg.content {
                blocks
                    .iter()
                    .filter_map(|b| {
                        if b.block_type.as_deref() == Some("text") {
                            b.text.clone()
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                String::new()
            };

            if !text.is_empty() {
                md.push_str(&format!("## {}\n\n{}\n\n---\n\n", label, text));
            }
        }
    }

    Ok(md)
}
