use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub post_count: u64,
    pub owner: Addr,
}

// Create PostData struct with relevant post information.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PostData {
    pub username: String,
    pub post_text: String,
    pub user_likes: Vec<String>,
}

// Create UserData struct with relevant user information.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UserData {
    pub username: String,
    pub blacklist: bool,
}

pub const STATE: Item<State> = Item::new("state");
pub const POSTDATA: Map<&str, PostData> = Map::new("post_data");
pub const USERDATA: Map<&Addr, UserData> = Map::new("user_data");
