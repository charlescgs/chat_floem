use std::collections::{BTreeMap, HashMap};
use std::cell::LazyCell;
use std::rc::Rc;

use floem::reactive::{create_memo, Scope};
use floem::ViewId;
use floem::{prelude::*, reactive::Memo};
use ulid::Ulid;

use crate::cont::acc;
use crate::views::msgs::RoomMsgUpt;
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
    pub rooms: RwSignal<BTreeMap<usize, RoomViewData>>,
    /// Extra data regarding rooms: index and ViewId.
    pub rooms_tabs: RwSignal<HashMap<Ulid, (usize, ViewId, RwSignal<RoomMsgUpt>)>>,
    pub rooms_tabs_count: Memo<usize>,
    /// An active room (if any).
    pub active_room: RwSignal<Option<RoomTabIdx>>,
    // /// The index of the active tab.
    // pub active_tab: RwSignal<usize>,
    pub common_data: Rc<CommonData>,
    // /// Stores info what range of its msgs is loaded.
    // pub active_room_msgs_data: RwSignal<RoomMsgChunks>,
    pub scope: Scope
}


impl UISession {
    pub fn new() -> Self {
        let cx = Scope::new();
        println!("UISession scope: {cx:#?}");
        let mut accs = Vec::with_capacity(3);
        while let Some(acc) = Account::new_from_click() {
            accs.push(acc);
        }
        let user = Rc::new(accs.remove(0));
        Self {
            user,
            accounts: cx.create_rw_signal(HashMap::from_iter(accs.into_iter().map(|acc| (acc.acc_id.id, acc)))),
            rooms: cx.create_rw_signal(BTreeMap::new()),
            rooms_tabs: cx.create_rw_signal(HashMap::new()),
            rooms_tabs_count: cx.create_memo(|_| 0),
            active_room: cx.create_rw_signal(None),
            common_data: Rc::new(CommonData::default()),
            scope: cx
        }
    }

    /// Returns scope related with [UISession] liftime.
    pub fn provide_scope(&self) -> Scope {
        self.scope
    }
}