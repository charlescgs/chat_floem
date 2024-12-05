use std::collections::{BTreeMap, HashMap};
use std::cell::LazyCell;
use std::rc::Rc;

use floem::reactive::create_memo;
use floem::ViewId;
use floem::{prelude::*, reactive::Memo};
use ulid::Ulid;

use crate::{common::CommonData, cont::acc::Account};
use super::room::{RoomTabIdx, RoomViewData};


thread_local! {
    /// Central structure for the UI thread.
    pub static APP: LazyCell<UISession> = LazyCell::new(UISession::new);
}



/// Contains all data needed to manage user session.
pub struct UISession {
    /// Account of the session user.
    pub user: Rc<Account>,
    /// List of all users.
    pub accounts: RwSignal<HashMap<Ulid, Account>>,
    /// List of all user rooms.
    pub rooms: RwSignal<HashMap<usize, RwSignal<RoomViewData>>>,
    /// Extra data regarding rooms: index and ViewId.
    pub rooms_tabs: RwSignal<HashMap<Ulid, (usize, ViewId)>>,
    pub rooms_tabs_count: Memo<usize>,
    /// An active room (if any).
    pub active_room: RwSignal<Option<RoomTabIdx>>,
    // /// The index of the active tab.
    // pub active_tab: RwSignal<usize>,
    pub common_data: Rc<CommonData>,
    // /// Stores info what range of its msgs is loaded.
    // pub active_room_msgs_data: RwSignal<RoomMsgChunks>,
}


impl UISession {
    pub fn new() -> Self {
        let user = Rc::new(Account::new_from_click().unwrap());
        Self {
            user,
            accounts: RwSignal::new(HashMap::new()),
            rooms: RwSignal::new(HashMap::new()),
            rooms_tabs: RwSignal::new(HashMap::new()),
            rooms_tabs_count: create_memo(|_| 0),
            active_room: RwSignal::new(None),
            common_data: Rc::new(CommonData::default())
        }
    }
}


#[derive(Clone, Debug)]
pub enum MsgUpdate {
    New {
        room: Ulid,
        msg: Ulid
    },
    Updated {
        room: Ulid,
        msg: Ulid
    },
    None
}