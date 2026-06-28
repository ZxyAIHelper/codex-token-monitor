use chrono::DateTime;
use serde::Serialize;

use crate::{codex_log::MessageDetail, usage_store::TurnDetail};

#[derive(Debug, Serialize)]
pub struct ModelRequestDetail {
    pub request_index: usize,
    pub turn: TurnDetail,
    pub messages: Vec<MessageDetail>,
}

pub fn group_model_requests(
    mut turns: Vec<TurnDetail>,
    mut messages: Vec<MessageDetail>,
) -> Vec<ModelRequestDetail> {
    turns.sort_by_key(|turn| timestamp_key(&turn.timestamp));
    messages.sort_by_key(|message| timestamp_key(&message.timestamp));

    let mut next_message_index = 0;
    turns
        .into_iter()
        .enumerate()
        .map(|(turn_index, turn)| {
            let turn_time = timestamp_key(&turn.timestamp);
            let mut request_messages = Vec::new();

            while next_message_index < messages.len()
                && timestamp_key(&messages[next_message_index].timestamp) <= turn_time
            {
                request_messages.push(messages[next_message_index].clone());
                next_message_index += 1;
            }

            ModelRequestDetail {
                request_index: turn_index + 1,
                turn,
                messages: request_messages,
            }
        })
        .collect()
}

fn timestamp_key(timestamp: &str) -> i64 {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|value| value.timestamp_millis())
        .unwrap_or(i64::MIN)
}
