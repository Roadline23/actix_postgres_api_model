use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SignUpDataResult {
    pub firstName: String,
    pub lastName: String,
    pub email: String,
    pub phone: String,
    pub address: Option<String>,
    pub postal: Option<String>,
    pub city: Option<String>,
    pub denomination: Option<String>,
    pub sameIdentity: bool,
    pub checkSiren: bool,
    pub siren: String,
    pub terms: bool,
    pub privacy: bool,
}