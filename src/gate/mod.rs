pub mod telegram;

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use tracing::info;

use crate::ledger::Ledger;
use crate::models::DraftStatus;
use telegram::{parse_command, GateCommand, TelegramGate};

#[derive(Deserialize)]
struct UpdatesResp {
    result: Vec<TgUpdate>,
}

#[derive(Deserialize)]
struct TgUpdate {
    update_id: i64,
    message:   Option<TgMessage>,
}

#[derive(Deserialize)]
struct TgMessage {
    text: Option<String>,
    chat: TgChat,
}

#[derive(Deserialize)]
struct TgChat {
    id: i64,
}

pub async fn process_updates(
    bot_token: &str,
    chat_id:   &str,
    ledger:    &Ledger,
    last_id:   &mut i64,
) -> Result<usize> {
    let client = Client::new();
    let gate   = TelegramGate::new(bot_token, chat_id);

    let url = format!(
        "https://api.telegram.org/bot{}/getUpdates?offset={}&timeout=10",
        bot_token,
        *last_id + 1,
    );

    let updates: Vec<TgUpdate> = client
        .get(&url)
        .send().await?
        .json::<UpdatesResp>().await?
        .result;

    let expected: i64 = chat_id.parse().unwrap_or(0);
    let mut count = 0usize;

    for upd in updates {
        *last_id = upd.update_id;

        let msg = match upd.message {
            Some(m) => m,
            None    => continue,
        };

        if msg.chat.id != expected {
            continue;
        }

        let text = match msg.text {
            Some(t) => t,
            None    => continue,
        };

        match parse_command(&text) {
            GateCommand::Approve(id) => {
                if ledger.update_draft_status(&id, DraftStatus::Approved).await? {
                    gate.notify(&format!(
                        "OK: {} approved and ready to dispatch.", id
                    )).await?;
                    count += 1;
                    info!("[Gate] Approved: {}", id);
                }
            }
            GateCommand::Reject(id) => {
                if ledger.update_draft_status(&id, DraftStatus::Rejected).await? {
                    gate.notify(&format!(
                        "REJECTED: {}.", id
                    )).await?;
                    count += 1;
                    info!("[Gate] Rejected: {}", id);
                }
            }
            GateCommand::Edit { id, new_content } => {
                if ledger.update_draft_content(&id, &new_content).await? {
                    ledger.update_draft_status(&id, DraftStatus::Approved).await?;
                    gate.notify(&format!(
                        "EDITED and APPROVED: {}.", id
                    )).await?;
                    count += 1;
                    info!("[Gate] Edited+Approved: {}", id);
                }
            }
            GateCommand::Unknown => {}
        }
    }

    Ok(count)
}
