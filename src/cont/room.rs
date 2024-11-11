use chrono_lite::Datetime;
use serde::{Deserialize, Serialize};

use crate::util::Id;
use super::acc::Member;




#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Room {
    #[serde(rename = "id")]
    pub room_id: Id,
    pub members: Vec<Member>,
    pub active_members: Vec<Id>,
    #[serde(default)]
    pub active_invites: Option<Vec<u8>>,
    pub owner: Id,
    pub created: Datetime,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub avatar: Option<Vec<u8>>,
}