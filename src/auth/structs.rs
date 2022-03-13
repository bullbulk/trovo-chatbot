use std::collections::HashMap;
use std::{fmt};

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshResponse {
    pub(crate) access_token: String,
    pub(crate) refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UsersResponse {
    pub(crate) users: Vec<HashMap<String, String>>,
}

#[derive(Debug)]
struct InvalidResponse;


impl fmt::Display for InvalidResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Caught an invalid response")
    }
}