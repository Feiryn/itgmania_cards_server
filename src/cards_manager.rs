use tokio::sync::Mutex;
use std::sync::OnceLock;

static PLAYER1_CARD_NUMBER: OnceLock<Mutex<Option<String>>> = OnceLock::new();
static PLAYER2_CARD_NUMBER: OnceLock<Mutex<Option<String>>> = OnceLock::new();
static CARDS_ENABLED: OnceLock<Mutex<bool>> = OnceLock::new();

fn get_player1_mutex() -> &'static Mutex<Option<String>> {
    PLAYER1_CARD_NUMBER.get_or_init(|| Mutex::new(None))
}

fn get_player2_mutex() -> &'static Mutex<Option<String>> {
    PLAYER2_CARD_NUMBER.get_or_init(|| Mutex::new(None))
}

fn get_enabled_mutex() -> &'static Mutex<bool> {
    CARDS_ENABLED.get_or_init(|| Mutex::new(false))
}

pub async fn get_current_card_number_player1() -> Option<String> {
    let card_number = get_player1_mutex().lock().await;
    card_number.clone()
}

pub async fn get_current_card_number_player2() -> Option<String> {
    let card_number = get_player2_mutex().lock().await;
    card_number.clone()
}

pub async fn set_current_card_number_player1(card_number: String) {
    if is_enabled().await {
        let mut card_number_lock = get_player1_mutex().lock().await;
        *card_number_lock = Some(card_number);
    }
}

pub async fn set_current_card_number_player2(card_number: String) {
    if is_enabled().await {
        let mut card_number_lock = get_player2_mutex().lock().await;
        *card_number_lock = Some(card_number);
    }
}

pub async fn clear_current_card_player1() {
    let mut card_number_lock = get_player1_mutex().lock().await;
    *card_number_lock = None;
}

pub async fn clear_current_card_player2() {
    let mut card_number_lock = get_player2_mutex().lock().await;
    *card_number_lock = None;
}

pub async fn is_enabled() -> bool {
    let enabled = get_enabled_mutex().lock().await;
    *enabled
}

pub async fn set_enabled(enabled: bool) {
    let mut enabled_lock = get_enabled_mutex().lock().await;
    *enabled_lock = enabled;
    
    // If disabling, clear all cards
    if !enabled {
        drop(enabled_lock); // Release the lock before calling other functions
        clear_current_card_player1().await;
        clear_current_card_player2().await;
    }
}