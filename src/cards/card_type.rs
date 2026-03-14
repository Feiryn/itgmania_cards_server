#[derive(Debug, Clone)]
pub enum CardType {
    Felica,
    Mifare,
}

impl TryFrom<String> for CardType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "felica" => Ok(CardType::Felica),
            "mifare" => Ok(CardType::Mifare),
            _ => Err(format!("Unknown card type: {}", value)),
        }
    }
}

impl ToString for CardType {
    fn to_string(&self) -> String {
        match self {
            CardType::Felica => "FELICA".to_string(),
            CardType::Mifare => "MIFARE".to_string(),
        }
    }
}
