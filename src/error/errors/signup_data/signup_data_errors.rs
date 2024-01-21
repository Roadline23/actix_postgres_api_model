use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct SignUpDataErrors {
    pub firstName: String,
    pub lastName: String,
    pub email: String,
    pub phone: String,
    pub spe: String,
    pub siren: String,
    pub terms: String,
    pub privacy: String,
}

impl SignUpDataErrors {
    pub fn new() -> Self {
        SignUpDataErrors {
            firstName: String::new(),
            lastName: String::new(),
            email: String::new(),
            phone: String::new(),
            spe: String::new(),
            siren: String::new(),
            terms: String::new(),
            privacy: String::new(),
        }
    }
}
