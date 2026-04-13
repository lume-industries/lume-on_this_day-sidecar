use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventRow {
    pub year: String,
    pub event: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OnThisDayPayload {
    pub date_label: String,
    pub updated: String,
    pub events: Vec<EventRow>,
}

#[derive(Deserialize)]
struct ApiResponse {
    #[serde(default)]
    events: Vec<ApiEvent>,
}

#[derive(Deserialize)]
struct ApiEvent {
    year: Option<i32>,
    text: Option<String>,
}

fn truncate_at_word(text: &str, max_len: usize) -> String {
    if text.chars().count() <= max_len {
        return text.to_string();
    }
    let mut candidate = String::new();
    for ch in text.chars().take(max_len.saturating_sub(3)) {
        candidate.push(ch);
    }
    if let Some(idx) = candidate.rfind(' ') {
        candidate.truncate(idx);
    }
    candidate.push_str("...");
    candidate
}

pub fn parse_events_payload(
    body: &str,
    date_label: String,
    now_secs: u64,
) -> Result<OnThisDayPayload, String> {
    const MAX_EVENTS: usize = 7;
    const EVENT_MAX_LEN: usize = 34;

    let mut response: ApiResponse =
        serde_json::from_str(body).map_err(|error| format!("invalid On This Day JSON: {error}"))?;
    response
        .events
        .sort_by_key(|event| std::cmp::Reverse(event.year));

    let events: Vec<EventRow> = response
        .events
        .into_iter()
        .filter_map(|event| {
            let year = event.year?;
            let text = event.text?;
            let text = text.trim().to_string();
            if text.is_empty() {
                return None;
            }
            Some(EventRow {
                year: year.to_string(),
                event: truncate_at_word(&text, EVENT_MAX_LEN),
            })
        })
        .take(MAX_EVENTS)
        .collect();

    if events.is_empty() {
        return Err("Wikipedia On This Day returned no usable events".to_string());
    }

    let hh = (now_secs % 86400) / 3600;
    let mm = (now_secs % 3600) / 60;
    Ok(OnThisDayPayload {
        date_label,
        updated: format!("Updated {:02}:{:02}", hh, mm),
        events,
    })
}
