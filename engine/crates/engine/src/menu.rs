//! Demo USSD menu walker.
//!
//! Carriers send the *cumulative* `text` field ("1", then "1*2", then "1*2*500")
//! on every callback, so the walker reconstructs progress from that cumulative
//! input. It also reads and writes session state so it can validate that a step
//! arrived in the expected menu context — the same read path the schema-driven
//! walker will use once the menu DSL lands.

use std::time::Duration;

use crate::session::SessionStore;

/// A reply the engine sends back to the carrier.
#[derive(Debug, Clone)]
pub struct MenuReply {
    pub text: String,
    pub is_end: bool,
}

impl MenuReply {
    /// Format as raw USSD response: "CON ..." keeps the session open,
    /// "END ..." terminates it.
    pub fn to_body(self) -> String {
        let prefix = if self.is_end { "END " } else { "CON " };
        format!("{prefix}{}", self.text)
    }

    fn con(text: impl Into<String>) -> Self {
        Self { text: text.into(), is_end: false }
    }

    fn end(text: impl Into<String>) -> Self {
        Self { text: text.into(), is_end: true }
    }
}

const MAIN_MENU: &str = "Welcome to KagoRoute demo!\n1. Check balance\n2. Buy airtime\n0. Exit";

/// Walk the demo menu for one callback. `text` is the cumulative input.
pub async fn run_demo_menu(
    store: &SessionStore,
    session_id: &str,
    phone_number: &str,
    text: &str,
) -> MenuReply {
    let key = format!("session:{session_id}");

    match text {
        // First screen of the session.
        "" => {
            store.set(&key, "menu:main", Duration::from_secs(120)).await;
            MenuReply::con(MAIN_MENU)
        }

        "0" => {
            store.delete(&key).await;
            MenuReply::end("Goodbye! You are now logged out.")
        }

        "1" => {
            store.delete(&key).await;
            MenuReply::end(format!(
                "Your balance is KES 1,250.00.\nThank you for using KagoRoute, {phone_number}."
            ))
        }

        // Enter airtime amount.
        "2" => {
            store.set(&key, "menu:airtime", Duration::from_secs(120)).await;
            MenuReply::con("Enter airtime amount in KES:")
        }

        // "2*<amount>" — only accept an amount if this session is actually in
        // the airtime menu; guards against stale or out-of-context inputs.
        input if input.starts_with("2*") => {
            let in_airtime_menu = store.get(&key).await.as_deref() == Some("menu:airtime");
            if !in_airtime_menu {
                store.delete(&key).await;
                return MenuReply::end("Session expired. Please dial *123# to start again.");
            }

            let amount = &input[2..];
            if !amount.is_empty() && amount.chars().all(|c| c.is_ascii_digit()) {
                store.delete(&key).await;
                MenuReply::end(format!(
                    "Airtime of KES {amount} for {phone_number} queued for processing.\nYou will receive a confirmation SMS shortly."
                ))
            } else {
                MenuReply::con("Invalid amount. Enter digits only, e.g. 2*500:")
            }
        }

        _ => {
            store.set(&key, "menu:main", Duration::from_secs(120)).await;
            MenuReply::con(format!("Invalid option. Please try again.\n{MAIN_MENU}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::memory::MemoryStore;

    #[tokio::test]
    async fn first_screen_opens_session_with_con() {
        let store = SessionStore::Memory(MemoryStore::default());
        let reply = run_demo_menu(&store, "s1", "254712345678", "").await;
        assert!(!reply.is_end);
        let body = reply.to_body();
        assert!(body.starts_with("CON "));
        assert!(body.contains("Check balance"));
    }

    #[tokio::test]
    async fn balance_option_terminates_with_end() {
        let store = SessionStore::Memory(MemoryStore::default());
        let reply = run_demo_menu(&store, "s1", "254712345678", "1").await;
        assert!(reply.is_end);
        assert!(reply.text.contains("1,250.00"));
        assert!(reply.to_body().starts_with("END "));
    }

    #[tokio::test]
    async fn airtime_flow_accumulates_input() {
        let store = SessionStore::Memory(MemoryStore::default());
        let reply = run_demo_menu(&store, "s1", "254712345678", "2").await;
        assert!(!reply.is_end);

        let reply = run_demo_menu(&store, "s1", "254712345678", "2*500").await;
        assert!(reply.is_end);
        assert!(reply.text.contains("KES 500"));
    }

    #[tokio::test]
    async fn invalid_option_keeps_session_open() {
        let store = SessionStore::Memory(MemoryStore::default());
        let reply = run_demo_menu(&store, "s1", "254712345678", "9").await;
        assert!(!reply.is_end);
    }
}
