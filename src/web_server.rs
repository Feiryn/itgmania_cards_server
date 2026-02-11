use rocket::{get, post, routes, uri, Build, State};
use rocket::form::Form;
use rocket::response::{content::RawHtml, Redirect, Flash};
use rocket::http::CookieJar;
use rocket::request::FlashMessage;
use rocket::FromForm;
use rocket::fs::{NamedFile, TempFile};
use std::path::PathBuf;
use crate::auth::{AuthenticatedUser, verify_password, set_auth_cookie, remove_auth_cookie};
use crate::config::Config;
use crate::templates::Templates;
use crate::accounts::{self, UpdateAccountDetails};
use crate::cards_manager;

#[derive(FromForm)]
struct LoginForm {
    password: String,
}

#[derive(FromForm)]
struct CreateAccountForm<'r> {
    card_number: String,
    display_name: String,
    display_name_four_letters: String,
    groovestats_api_key: Option<String>,
    avatar: Option<TempFile<'r>>,
}

#[derive(FromForm)]
struct UpdateAccountForm<'r> {
    old_card_number: String,
    display_name: String,
    display_name_four_letters: String,
    groovestats_api_key: Option<String>,
    avatar: Option<TempFile<'r>>,
}

#[derive(FromForm)]
struct DeleteAccountForm {
    card_number: String,
}

#[derive(FromForm)]
struct InsertCardForm {
    card_number: String,
    player: u8,
}

#[derive(FromForm)]
struct RemoveCardForm {
    player: u8,
}

async fn render_home_page(templates: &Templates, message: Option<(&str, bool)>) -> RawHtml<String> {
    let accounts = accounts::list_accounts();
    let account_count = accounts.len();

    // Render player slots
    let player1_card = cards_manager::get_current_card_number_player1().await;
    let player2_card = cards_manager::get_current_card_number_player2().await;
    let cards_enabled = cards_manager::is_enabled().await;

    let player1_content = if let Some(card_num) = &player1_card {
        if let Some(details) = accounts::get_account_details(card_num) {
            format!(
                r#"<div class="slot-card">
                    <strong>{}</strong>
                    <small>Card: {}</small>
                </div>
                <form method="post" action="/cards/remove">
                    <input type="hidden" name="player" value="1">
                    <button type="submit" class="btn btn-danger btn-small" style="width: 100%;">Remove Card</button>
                </form>"#,
                details.display_name, details.card_number
            )
        } else {
            r#"<div class="slot-empty">No card inserted</div>"#.to_string()
        }
    } else {
        r#"<div class="slot-empty">No card inserted</div>"#.to_string()
    };

    let player2_content = if let Some(card_num) = &player2_card {
        if let Some(details) = accounts::get_account_details(card_num) {
            format!(
                r#"<div class="slot-card">
                    <strong>{}</strong>
                    <small>Card: {}</small>
                </div>
                <form method="post" action="/cards/remove">
                    <input type="hidden" name="player" value="2">
                    <button type="submit" class="btn btn-danger btn-small" style="width: 100%;">Remove Card</button>
                </form>"#,
                details.display_name, details.card_number
            )
        } else {
            r#"<div class="slot-empty">No card inserted</div>"#.to_string()
        }
    } else {
        r#"<div class="slot-empty">No card inserted</div>"#.to_string()
    };

    // Render message if present
    let message_html = if let Some((msg, is_success)) = message {
        let class = if is_success { "message-success" } else { "message-error" };
        format!(r#"<div class="message {}">{}</div>"#, class, msg)
    } else {
        String::new()
    };

    // Render accounts table or empty state
    let accounts_content = if accounts.is_empty() {
        r#"<div class="empty-state">
            <svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" fill="currentColor">
                <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z"/>
            </svg>
            <h3>No Accounts Yet</h3>
            <p>Get started by creating your first account.</p>
            <button class="btn" onclick="openAddModal()">+ Create First Account</button>
        </div>"#.to_string()
    } else {
        let mut table_rows = String::new();
        for account in accounts {
            let gs_badge = if account.has_groovestats_api_key {
                r#"<span class="badge badge-success">Present</span>"#
            } else {
                r#"<span class="badge badge-secondary">None</span>"#
            };

            let is_in_p1 = player1_card.as_ref().map_or(false, |c| c == &account.card_number);
            let is_in_p2 = player2_card.as_ref().map_or(false, |c| c == &account.card_number);

            let insert_buttons = if is_in_p1 || is_in_p2 {
                format!(r#"<span style="font-size: 11px; color: #4caf50;">✓ In use (P{})</span>"#, if is_in_p1 { "1" } else { "2" })
            } else if !cards_enabled {
                r#"<span style="font-size: 11px; color: #999;">🔒 Cards disabled</span>"#.to_string()
            } else {
                format!(
                    r#"<form method="post" action="/cards/insert" style="display: inline;">
                        <input type="hidden" name="card_number" value="{}">
                        <input type="hidden" name="player" value="1">
                        <button type="submit" class="btn btn-success btn-small">→ P1</button>
                    </form>
                    <form method="post" action="/cards/insert" style="display: inline;">
                        <input type="hidden" name="card_number" value="{}">
                        <input type="hidden" name="player" value="2">
                        <button type="submit" class="btn btn-success btn-small">→ P2</button>
                    </form>"#,
                    account.card_number, account.card_number
                )
            };
            
            let avatar_html = if account.avatar_path.is_some() {
                format!(r#"<img src="/avatars/{}" alt="Avatar" class="avatar-img">"#, account.card_number)
            } else {
                r#"<div class="avatar-placeholder">👤</div>"#.to_string()
            };

            table_rows.push_str(&format!(
                r#"<tr>
                    <td>{}</td>
                    <td><strong>{}</strong></td>
                    <td>{}</td>
                    <td><code>{}</code></td>
                    <td>{}</td>
                    <td><small>{}</small></td>
                    <td>{}</td>
                    <td>
                        <div class="actions">
                            <button class="btn btn-small" onclick="openEditModal('{}', '{}', '{}')">✏️ Edit</button>
                            <button class="btn btn-danger btn-small" onclick="deleteAccount('{}', '{}')">🗑️ Delete</button>
                        </div>
                    </td>
                </tr>"#,
                avatar_html,
                account.display_name,
                account.display_name_four_letters,
                account.card_number,
                gs_badge,
                account.last_played_date,
                insert_buttons,
                escape_quotes(&account.card_number),
                escape_quotes(&account.display_name),
                escape_quotes(&account.display_name_four_letters),
                escape_quotes(&account.card_number),
                escape_quotes(&account.display_name)
            ));
        }

        format!(
            r#"<table class="accounts-table">
                <thead>
                    <tr>
                        <th>Avatar</th>
                        <th>Display Name</th>
                        <th>Short</th>
                        <th>Card Number</th>
                        <th>GrooveStats API Key</th>
                        <th>Last Played</th>
                        <th>Insert Card</th>
                        <th>Actions</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>"#,
            table_rows
        )
    };

    let html = templates.home
        .replace("PLAYER1_CONTENT", &player1_content)
        .replace("PLAYER2_CONTENT", &player2_content)
        .replace("MESSAGE_PLACEHOLDER", &message_html)
        .replace("ACCOUNT_COUNT", &account_count.to_string())
        .replace("ACCOUNTS_CONTENT", &accounts_content);

    RawHtml(html)
}

fn escape_quotes(s: &str) -> String {
    s.replace('\'', "\\'").replace('"', "&quot;")
}

#[get("/")]
async fn index(_user: AuthenticatedUser, templates: &State<Templates>, flash: Option<FlashMessage<'_>>) -> RawHtml<String> {
    let message = flash.map(|flash| {
        let is_success = flash.kind() == "success";
        (flash.message().to_string(), is_success)
    });
    render_home_page(templates, message.as_ref().map(|(msg, success)| (msg.as_str(), *success))).await
}

#[get("/", rank = 2)]
fn index_redirect() -> Redirect {
    Redirect::to(uri!("/login"))
}

#[get("/login")]
fn login_page(_user: AuthenticatedUser) -> Redirect {
    Redirect::to(uri!("/"))
}

#[get("/login", rank = 2)]
fn login_page_unauthenticated(templates: &State<Templates>) -> RawHtml<String> {
    RawHtml(templates.login.replace("ERROR_PLACEHOLDER", ""))
}

#[post("/login", data = "<form>")]
fn login_submit(
    form: Form<LoginForm>,
    cookies: &CookieJar<'_>,
    config: &State<Config>,
    templates: &State<Templates>,
) -> Result<Redirect, RawHtml<String>> {
    if verify_password(&form.password, &config.auth.password_hash) {
        set_auth_cookie(cookies);
        Ok(Redirect::to(uri!("/")))
    } else {
        let error_html = r#"<div class="error-message">❌ Invalid password. Please try again.</div>"#;
        Err(RawHtml(templates.login.replace("ERROR_PLACEHOLDER", error_html)))
    }
}

#[get("/logout")]
fn logout(cookies: &CookieJar<'_>) -> Redirect {
    remove_auth_cookie(cookies);
    Redirect::to(uri!("/login"))
}

#[post("/accounts/create", data = "<form>")]
async fn create_account(
    _user: AuthenticatedUser,
    mut form: Form<CreateAccountForm<'_>>,
) -> Flash<Redirect> {
    // Validate card number format
    let card_number = form.card_number.trim().to_uppercase();
    if card_number.is_empty() || card_number.len() < 8 || card_number.len() > 16 {
        return Flash::error(Redirect::to(uri!("/")), "Invalid card number. Must be 8-16 hexadecimal characters.");
    }

    if !card_number.chars().all(|c| c.is_ascii_hexdigit()) {
        return Flash::error(Redirect::to(uri!("/")), "Invalid card number. Only hexadecimal characters (0-9, A-F) are allowed.");
    }

    // Validate display name
    let display_name = form.display_name.trim().to_string();
    if display_name.is_empty() {
        return Flash::error(Redirect::to(uri!("/")), "Display name cannot be empty.");
    }

    // Validate short name
    let short_name = form.display_name_four_letters.trim().to_uppercase();
    if short_name.is_empty() || short_name.len() > 4 || !short_name.chars().all(|c| c.is_ascii_alphabetic()) {
        return Flash::error(Redirect::to(uri!("/")), "Short name must be 1-4 letters.");
    }

    // Check if account already exists
    if accounts::does_account_exist(&card_number) {
        return Flash::error(Redirect::to(uri!("/")), "An account with this card number already exists.");
    }
    
    // Get groovestats key
    let gs_key = form.groovestats_api_key.as_ref().map(|s| s.trim()).unwrap_or("").to_string();
    
    // Process avatar if provided
    let (avatar_data, avatar_ext) = if let Some(ref mut avatar) = form.avatar {
        if avatar.len() > 0 {
            // Get file extension from content type or filename
            let extension = avatar.content_type()
                .and_then(|ct| match ct.to_string().as_str() {
                    "image/png" => Some("png".to_string()),
                    "image/jpeg" => Some("jpg".to_string()),
                    "image/jpg" => Some("jpg".to_string()),
                    "image/bmp" => Some("bmp".to_string()),
                    "image/gif" => Some("gif".to_string()),
                    _ => None,
                })
                .or_else(|| {
                    avatar.name().and_then(|name| {
                        name.split('.').last().and_then(|ext| {
                            let ext_lower = ext.to_lowercase();
                            if ["png", "jpg", "jpeg", "bmp", "gif"].contains(&ext_lower.as_str()) {
                                Some(if ext_lower == "jpeg" { "jpg".to_string() } else { ext_lower })
                            } else {
                                None
                            }
                        })
                    })
                });
            
            if let Some(ext) = extension {
                if let Some(path) = avatar.path() {
                    match tokio::fs::read(path).await {
                        Ok(data) => (Some(data), Some(ext)),
                        Err(e) => {
                            return Flash::error(Redirect::to(uri!("/")), format!("Failed to read avatar file: {}", e));
                        }
                    }
                } else {
                    return Flash::error(Redirect::to(uri!("/")), "Failed to access avatar file path.");
                }
            } else {
                return Flash::error(Redirect::to(uri!("/")), "Unsupported avatar format. Use png, jpg, jpeg, bmp, or gif.");
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // Create the account
    match accounts::create_new_account(
        &card_number, 
        &display_name, 
        &short_name, 
        &gs_key,
        avatar_data.as_deref(),
        avatar_ext.as_deref(),
    ) {
        Ok(_) => Flash::success(Redirect::to(uri!("/")), format!("Account '{}' created successfully!", display_name)),
        Err(e) => Flash::error(Redirect::to(uri!("/")), format!("Failed to create account: {}", e)),
    }
}

#[post("/accounts/update", data = "<form>")]
async fn update_account(
    _user: AuthenticatedUser,
    mut form: Form<UpdateAccountForm<'_>>,
) -> Flash<Redirect> {
    let card_number = form.old_card_number.trim().to_string();

    // Validate display name
    let display_name = form.display_name.trim().to_string();
    if display_name.is_empty() {
        return Flash::error(Redirect::to(uri!("/")), "Display name cannot be empty.");
    }

    // Validate short name
    let short_name = form.display_name_four_letters.trim().to_uppercase();
    if short_name.is_empty() || short_name.len() > 4 || !short_name.chars().all(|c| c.is_ascii_alphabetic()) {
        return Flash::error(Redirect::to(uri!("/")), "Short name must be 1-4 letters.");
    }

    // Check if account exists
    if !accounts::does_account_exist(&card_number) {
        return Flash::error(Redirect::to(uri!("/")), "Account not found.");
    }
    
    // Get groovestats key
    let groovestats_key = form.groovestats_api_key.as_ref().and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
    });
    
    // Process avatar if provided
    let (avatar_data, avatar_ext) = if let Some(ref mut avatar) = form.avatar {
        if avatar.len() > 0 {
            // Get file extension from content type or filename
            let extension = avatar.content_type()
                .and_then(|ct| match ct.to_string().as_str() {
                    "image/png" => Some("png".to_string()),
                    "image/jpeg" => Some("jpg".to_string()),
                    "image/jpg" => Some("jpg".to_string()),
                    "image/bmp" => Some("bmp".to_string()),
                    "image/gif" => Some("gif".to_string()),
                    _ => None,
                })
                .or_else(|| {
                    avatar.name().and_then(|name| {
                        name.split('.').last().and_then(|ext| {
                            let ext_lower = ext.to_lowercase();
                            if ["png", "jpg", "jpeg", "bmp", "gif"].contains(&ext_lower.as_str()) {
                                Some(if ext_lower == "jpeg" { "jpg".to_string() } else { ext_lower })
                            } else {
                                None
                            }
                        })
                    })
                });
            
            if let Some(ext) = extension {
                if let Some(path) = avatar.path() {
                    match tokio::fs::read(path).await {
                        Ok(data) => (Some(data), Some(ext)),
                        Err(e) => {
                            return Flash::error(Redirect::to(uri!("/")), format!("Failed to read avatar file: {}", e));
                        }
                    }
                } else {
                    return Flash::error(Redirect::to(uri!("/")), "Failed to access avatar file path.");
                }
            } else {
                return Flash::error(Redirect::to(uri!("/")), "Unsupported avatar format. Use png, jpg, jpeg, bmp, or gif.");
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    // Update the account
    let updates = UpdateAccountDetails {
        display_name: Some(display_name.clone()),
        display_name_four_letters: Some(short_name),
        groovestats_api_key: groovestats_key,
        avatar_data,
        avatar_extension: avatar_ext,
    };

    match accounts::update_account_details(&card_number, updates) {
        Ok(_) => Flash::success(Redirect::to(uri!("/")), format!("Account '{}' updated successfully!", display_name)),
        Err(e) => Flash::error(Redirect::to(uri!("/")), format!("Failed to update account: {}", e)),
    }
}

#[post("/accounts/delete", data = "<form>")]
async fn delete_account(
    _user: AuthenticatedUser,
    form: Form<DeleteAccountForm>,
) -> Flash<Redirect> {
    let card_number = form.card_number.trim();

    // Check if account exists
    if !accounts::does_account_exist(card_number) {
        return Flash::error(Redirect::to(uri!("/")), "Account not found.");
    }

    // Get account name before deletion
    let account_name = accounts::get_account_details(card_number)
        .map(|a| a.display_name)
        .unwrap_or_else(|| card_number.to_string());

    // Delete the account
    match accounts::delete_account(card_number) {
        Ok(_) => Flash::success(Redirect::to(uri!("/")), format!("Account '{}' deleted successfully!", account_name)),
        Err(e) => Flash::error(Redirect::to(uri!("/")), format!("Failed to delete account: {}", e)),
    }
}

#[post("/cards/insert", data = "<form>")]
async fn insert_card(
    _user: AuthenticatedUser,
    form: Form<InsertCardForm>,
) -> Flash<Redirect> {
    // Check if cards are enabled
    if !cards_manager::is_enabled().await {
        return Flash::error(Redirect::to(uri!("/")), "Card insertion is currently disabled.");
    }

    let card_number = form.card_number.trim();

    // Check if account exists
    if !accounts::does_account_exist(card_number) {
        return Flash::error(Redirect::to(uri!("/")), "Account not found.");
    }

    // Validate player number
    if form.player != 1 && form.player != 2 {
        return Flash::error(Redirect::to(uri!("/")), "Invalid player number. Must be 1 or 2.");
    }

    // Check if card is already inserted in any slot
    let p1_card = cards_manager::get_current_card_number_player1().await;
    let p2_card = cards_manager::get_current_card_number_player2().await;

    if p1_card.as_ref().map_or(false, |c| c == card_number) {
        return Flash::error(Redirect::to(uri!("/")), "This card is already inserted in Player 1 slot.");
    }
    if p2_card.as_ref().map_or(false, |c| c == card_number) {
        return Flash::error(Redirect::to(uri!("/")), "This card is already inserted in Player 2 slot.");
    }

    // Insert the card
    let account_name = accounts::get_account_details(card_number)
        .map(|a| a.display_name)
        .unwrap_or_else(|| card_number.to_string());

    if form.player == 1 {
        cards_manager::set_current_card_number_player1(card_number.to_string()).await;
        Flash::success(Redirect::to(uri!("/")), format!("Card '{}' inserted into Player 1 slot!", account_name))
    } else {
        cards_manager::set_current_card_number_player2(card_number.to_string()).await;
        Flash::success(Redirect::to(uri!("/")), format!("Card '{}' inserted into Player 2 slot!", account_name))
    }
}

#[post("/cards/remove", data = "<form>")]
async fn remove_card(
    _user: AuthenticatedUser,
    form: Form<RemoveCardForm>,
) -> Flash<Redirect> {
    // Validate player number
    if form.player != 1 && form.player != 2 {
        return Flash::error(Redirect::to(uri!("/")), "Invalid player number. Must be 1 or 2.");
    }

    if form.player == 1 {
        cards_manager::clear_current_card_player1().await;
        Flash::success(Redirect::to(uri!("/")), "Card removed from Player 1 slot.")
    } else {
        cards_manager::clear_current_card_player2().await;
        Flash::success(Redirect::to(uri!("/")), "Card removed from Player 2 slot.")
    }
}

#[get("/avatars/<card_number>")]
async fn get_avatar(card_number: &str) -> Option<NamedFile> {
    let avatar_path = accounts::get_avatar_path(&card_number)?;
    NamedFile::open(PathBuf::from(avatar_path)).await.ok()
}

pub fn build_rocket() -> rocket::Rocket<Build> {
    let config = Config::load().expect("Failed to load config.toml");
    let templates = Templates::load().expect("Failed to load templates");

    rocket::build()
        .manage(config)
        .manage(templates)
        .mount("/", routes![
            index,
            index_redirect,
            login_page,
            login_page_unauthenticated,
            login_submit,
            logout,
            create_account,
            update_account,
            delete_account,
            insert_card,
            remove_card,
            get_avatar,
        ])
}