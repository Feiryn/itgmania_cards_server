use std::fs;

pub struct Templates {
    pub login: String,
    pub home: String,
}

impl Templates {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let login = fs::read_to_string("templates/login.html")?;
        let home = fs::read_to_string("templates/home.html")?;

        Ok(Templates { login, home })
    }
}
