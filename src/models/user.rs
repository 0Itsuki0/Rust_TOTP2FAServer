use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub email: String,
    pub password: String,

    // base32 encoded secret string
    pub otp_secret: Option<String>,
    pub otp_verified: Option<bool>,
}

impl User {
    pub fn to_response_value(&self) -> Value {
        return json!({
            "email": self.email,
            "otp_enabled": self.otp_verified == Some(true)
        });
    }
}
