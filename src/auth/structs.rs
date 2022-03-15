use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
}
