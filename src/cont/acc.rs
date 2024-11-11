use chrono_lite::Datetime;
use serde::{Deserialize, Serialize};

use crate::util::Id;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Account {
    pub acc_id: Id,
    pub username: String,
    pub av: String,
    pub rooms: Vec<Id>
}

impl Account {
    fn new_from_click() -> Self {
        Self {
            acc_id: todo!(),
            username: todo!(),
            av: todo!(),
            rooms: todo!(),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Member {
    pub member_id: Id,
    pub since: Datetime
}