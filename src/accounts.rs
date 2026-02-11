fn accounts_dir() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    format!("{}/.itgmania_cards/accounts", home)
}

const NEW_ACCOUNT_EDITABLE_INI: &str = r#"[Editable]
BirthYear=0
CharacterID=
DisplayName={DISPLAY_NAME}
IgnoreStepCountCalories=0
IsMale=1
LastUsedHighScoreName={DISPLAY_NAME_FOUR_LETTERS}
Voomax=0.000000
WeightPounds=0"#;

const NEW_ACCOUNT_GROOVESTATS_INI: &str = r#"[GrooveStats]
ApiKey={GROOVESTATS_API_KEY}
IsPadPlayer=1"#;

const NEW_ACCOUNT_SIMPLY_LOVE_USER_PREFS_INI: &str = r#"[Simply Love]
ActionOnMissedTarget=Nothing
BackgroundFilter=Off
ColumnCues=false
ColumnFlashOnMiss=false
ComboFont=Wendy
DataVisualizations=None
DisplayScorebox=true
ErrorBar=None
ErrorBarMultiTick=false
ErrorBarTrim=Off
ErrorBarUp=false
HideCombo=false
HideComboExplosions=false
HideDanger=false
HideEarlyDecentWayOffFlash=false
HideEarlyDecentWayOffJudgments=false
HideLifebar=false
HideLookahead=false
HideScore=false
HideSongBG=false
HideTargets=false
HoldJudgment=Love 1x2 (doubleres).png
JudgmentGraphic=Love 2x7 (doubleres).png
JudgmentTilt=false
LifeMeterType=Standard
MeasureCounter=None
MeasureCounterLeft=true
MeasureCounterUp=false
MeasureLines=Off
Mini=0%
NPSGraphAtTop=false
NoteFieldOffsetX=0
NoteFieldOffsetY=0
NoteSkin=cel
Pacemaker=false
PlayerOptionsString=NoHideLights, m250, Overhead
ShowExScore=false
ShowFaPlusPane=true
ShowFaPlusWindow=false
SpeedMod=250
SpeedModType=M
SubtractiveScoring=false
TargetScore=11
TiltMultiplier=1
VisualDelay=0ms"#;

const NEW_ACCOUNT_STATS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" ?>
<Stats>
</Stats>"#;

const NEW_ACCOUNT_TYPE_INI: &str = r#"[ListPosition]
LastPlayedDate={LAST_PLAYED_DATE}
Priority=0
Type=Normal"#;

const NEW_ACCOUNT_EMPTY_FOLDERS: [&str; 5] = ["EditCourses", "Edits", "LastGood", "Rivals", "Screenshots"];

pub struct AccountDetails {
    pub card_number: String,
    pub display_name: String,
    pub display_name_four_letters: String,
    pub last_played_date: String,
    pub has_groovestats_api_key: bool,
    pub avatar_path: Option<String>,
}

pub struct UpdateAccountDetails {
    pub display_name: Option<String>,
    pub display_name_four_letters: Option<String>,
    pub groovestats_api_key: Option<String>,
    pub avatar_data: Option<Vec<u8>>,
    pub avatar_extension: Option<String>,
}

pub fn does_account_exist(card_number: &str) -> bool {
    if let Ok(entries) = std::fs::read_dir(accounts_dir()) {
        for entry in entries.flatten() {
            if entry.file_name() == card_number {
                return true;
            }
        }
    }

    false
}

pub fn save_avatar(card_number: &str, avatar_data: &[u8], extension: &str) -> std::io::Result<String> {
    let profile_path = format!("{}/{}/ITGmania", accounts_dir(), card_number);
    std::fs::create_dir_all(&profile_path)?;
    
    // Remove any existing avatar files
    let extensions = ["png", "jpg", "jpeg", "bmp", "gif"];
    for ext in extensions.iter() {
        let old_avatar_path = format!("{}/avatar.{}", profile_path, ext);
        let _ = std::fs::remove_file(&old_avatar_path); // Ignore if file doesn't exist
    }
    
    let avatar_path = format!("{}/avatar.{}", profile_path, extension);
    std::fs::write(&avatar_path, avatar_data)?;
    Ok(format!("avatar.{}", extension))
}

pub fn get_avatar_path(card_number: &str) -> Option<String> {
    let profile_path = format!("{}/{}/ITGmania", accounts_dir(), card_number);
    let extensions = ["png", "jpg", "jpeg", "bmp", "gif"];
    
    for ext in extensions.iter() {
        let avatar_path = format!("{}/avatar.{}", profile_path, ext);
        if std::fs::metadata(&avatar_path).is_ok() {
            return Some(avatar_path);
        }
    }
    
    None
}

pub fn create_new_account(card_number: &str, display_name: &str, display_name_four_letters: &str, groovestats_api_key: &str, avatar_data: Option<&[u8]>, avatar_extension: Option<&str>) -> std::io::Result<()> {
    let account_path = format!("{}/{}/ITGmania", accounts_dir(), card_number);
    std::fs::create_dir_all(&account_path)?;
    
    // Save avatar if provided
    if let (Some(data), Some(ext)) = (avatar_data, avatar_extension) {
        save_avatar(card_number, data, ext)?;
    }

    std::fs::write(format!("{}/Editable.ini", account_path), NEW_ACCOUNT_EDITABLE_INI.replace("{DISPLAY_NAME}", display_name).as_str().replace("{DISPLAY_NAME_FOUR_LETTERS}", display_name_four_letters))?;
    std::fs::write(format!("{}/GrooveStats.ini", account_path), NEW_ACCOUNT_GROOVESTATS_INI.replace("{GROOVESTATS_API_KEY}", groovestats_api_key))?;
    std::fs::write(format!("{}/Simply Love UserPrefs.ini", account_path), NEW_ACCOUNT_SIMPLY_LOVE_USER_PREFS_INI)?;
    std::fs::write(format!("{}/Stats.xml", account_path), NEW_ACCOUNT_STATS_XML)?;
    std::fs::write(format!("{}/Type.ini", account_path), NEW_ACCOUNT_TYPE_INI.replace("{LAST_PLAYED_DATE}", &chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()))?;

    for folder in NEW_ACCOUNT_EMPTY_FOLDERS.iter() {
        std::fs::create_dir_all(format!("{}/{}", account_path, folder))?;
    }

    Ok(())
}

pub fn delete_account(card_number: &str) -> std::io::Result<()> {
    // Move the account to a "Deleted" folder instead of permanently deleting it, just in case
    let deleted_path = format!("{}/Deleted/{}", accounts_dir(), card_number);

    if std::fs::metadata(&deleted_path).is_ok() {
        // If an account with the same card number already exists in the Deleted folder, remove it first
        std::fs::remove_dir_all(&deleted_path)?;
    }

    std::fs::create_dir_all(format!("{}/Deleted", accounts_dir()))?;
    std::fs::rename(format!("{}/{}", accounts_dir(), card_number), deleted_path)?;
    Ok(())
}

pub fn get_account_details(card_number: &str) -> Option<AccountDetails> {
    let account_path = format!("{}/{}/ITGmania", accounts_dir(), card_number);
    let editable_ini_path = format!("{}/Editable.ini", account_path);
    let type_ini_path = format!("{}/Type.ini", account_path);
    let groovestats_ini_path = format!("{}/GrooveStats.ini", account_path);

    if let (Ok(editable_ini), Ok(type_ini), Ok(groovestats_ini)) = (std::fs::read_to_string(editable_ini_path), std::fs::read_to_string(type_ini_path), std::fs::read_to_string(groovestats_ini_path)) {
        let display_name = editable_ini.as_str().lines().find(|line| line.starts_with("DisplayName=")).and_then(|line| line.split('=').nth(1)).unwrap_or("").to_string();
        let display_name_four_letters = editable_ini.as_str().lines().find(|line| line.starts_with("LastUsedHighScoreName=")).and_then(|line| line.split('=').nth(1)).unwrap_or("").to_string();
        let last_played_date = type_ini.as_str().lines().find(|line| line.starts_with("LastPlayedDate=")).and_then(|line| line.split('=').nth(1)).unwrap_or("").to_string();
        let has_groovestats_api_key = groovestats_ini.as_str().lines().find(|line| line.starts_with("ApiKey=")).and_then(|line| line.split('=').nth(1)).map_or(false, |key| !key.trim().is_empty());
        let avatar_path = get_avatar_path(card_number).map(|path| {
            // Return just the filename relative to the profile directory
            path.split('/').last().unwrap_or("").to_string()
        });

        return Some(AccountDetails {
            card_number: card_number.to_string(),
            display_name,
            display_name_four_letters,
            last_played_date,
            has_groovestats_api_key,
            avatar_path,
        });
    }

    None
}

pub fn list_accounts() -> Vec<AccountDetails> {
    let mut accounts = Vec::new();

    if let Ok(entries) = std::fs::read_dir(accounts_dir()) {
        for entry in entries.flatten() {
            let file_name_os = entry.file_name();
            if let Some(file_name) = file_name_os.as_os_str().to_str() {
                if let Some(details) = get_account_details(file_name) {
                    accounts.push(details);
                }
            }
        }
    }

    accounts
}

pub fn update_account_details(card_number: &str, updates: UpdateAccountDetails) -> std::io::Result<()> {
    let account_path = format!("{}/{}/ITGmania", accounts_dir(), card_number);
    let editable_ini_path = format!("{}/Editable.ini", account_path);
    let groovestats_ini_path = format!("{}/GrooveStats.ini", account_path);

    if let Ok(editable_ini) = std::fs::read_to_string(&editable_ini_path) {
        let mut updated_ini = editable_ini.clone();
        
        if let Some(display_name) = updates.display_name {
            // Extract current display name from INI
            if let Some(current) = editable_ini.lines()
                .find(|line| line.starts_with("DisplayName="))
                .and_then(|line| line.split('=').nth(1)) {
                updated_ini = updated_ini.replace(
                    &format!("DisplayName={}", current),
                    &format!("DisplayName={}", display_name),
                );
            }
        }
        
        if let Some(display_name_four_letters) = updates.display_name_four_letters {
            // Extract current short name from INI
            if let Some(current) = editable_ini.lines()
                .find(|line| line.starts_with("LastUsedHighScoreName="))
                .and_then(|line| line.split('=').nth(1)) {
                updated_ini = updated_ini.replace(
                    &format!("LastUsedHighScoreName={}", current),
                    &format!("LastUsedHighScoreName={}", display_name_four_letters),
                );
            }
        }
        
        std::fs::write(editable_ini_path, updated_ini)?;
    }

    if let Ok(groovestats_ini) = std::fs::read_to_string(&groovestats_ini_path) {
        if let Some(groovestats_api_key) = updates.groovestats_api_key {
            let updated_ini = groovestats_ini
                .lines()
                .map(|line| {
                    if line.starts_with("ApiKey=") {
                        format!("ApiKey={}", groovestats_api_key)
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<String>>()
                .join("\n");
            std::fs::write(groovestats_ini_path, updated_ini)?;
        }
    }
    
    // Update avatar if provided
    if let (Some(data), Some(ext)) = (updates.avatar_data, updates.avatar_extension) {
        save_avatar(card_number, &data, &ext)?;
    }

    Ok(())
}