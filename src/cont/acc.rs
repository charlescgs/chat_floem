use std::{rc::Rc, sync::atomic::{AtomicU8, AtomicUsize, Ordering}};

use chrono_lite::Datetime;
use serde::{Deserialize, Serialize};

use crate::util::{Id, Tb};


pub(super) static ACC_COUNTER: AtomicU8 = AtomicU8::new(0);



#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub acc_id: Id,
    pub username: String,
    pub av: Rc<Vec<u8>>,
    // pub rooms: Vec<Id>
}

impl Account {
    pub fn new_from_click() -> Option<Self> {
        if let Some((us, av)) = get_username_and_avatar() {
            Some(Self {
                acc_id: Id::new(Tb::Acc),
                username: us,
                av: Rc::new(av)
            })
        } else { None }
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Member {
    pub member_id: Id,
    pub since: Datetime
}


fn get_username_and_avatar() -> Option<(String, Vec<u8>)> {
    // static ACCS: [&str; 4] = ["Miśka", "Adam", "Robert", "Max"];
    // static IDX: AtomicUsize = AtomicUsize::new(0);
    match ACC_COUNTER.fetch_add(1, Ordering::Relaxed) {
        0 => Some(("Karol".into(), include_bytes!("../../assets/karol.jpg").to_vec())),
        1 => Some(("Konrad".into(), include_bytes!("../../assets/konrad.jpg").to_vec())),
        2 => Some(("Mama".into(), include_bytes!("../../assets/mama.jpg").to_vec())),
        _ => {
            // if IDX.load(Ordering::Relaxed) < 3 {
            //     Some((ACCS[IDX.fetch_add(1, Ordering::Relaxed)].into(), Vec::with_capacity(0)))
            // } else {
            //     IDX.store(0, Ordering::Relaxed);
            //     Some((ACCS[0].into(), Vec::with_capacity(0)))
            // }
            None
        }
    }
}