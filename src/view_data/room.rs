use std::cell::Cell;
use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use floem::reactive::{batch, create_effect, Memo, Trigger};
use floem::{prelude::*, ViewId};
use tracing_lite::{debug, trace, warn};
use ulid::Ulid;

use crate::cont::acc::Account;
use crate::util::{Id, Tb};
use crate::common::CommonData;
use crate::views::chunks::RoomMsgChunks;
use crate::views::msgs::RoomMsgUpt;

use super::msg::MsgViewData;
use super::session::APP;


pub static ROOM_IDX: AtomicUsize = AtomicUsize::new(0);


#[derive(Clone)]
/// Main structure containing all data needed for room view and msgs view.
pub struct RoomViewData {
    pub view_id: ViewId,
    pub room_id: Id,
    pub room_idx: RoomTabIdx,

    pub owner: Account,
    pub members: HashMap<Ulid, Account>,
    // pub msgs_count: Memo<u16>,
    pub msgs_count: Cell<u16>,
    pub description: RwSignal<Option<String>>,
    pub msgs: RwSignal<RoomMsgChunks>,
    
    pub get_update: RwSignal<RoomMsgUpt>,
    pub is_active: RwSignal<Cell<bool>>,
    pub last_msg: RwSignal<Option<MsgViewData>>,
    // pub display_state: RwSignal<DisplayState>,
    pub unread: RwSignal<bool>,
    pub num_unread: RwSignal<u16>,

    pub common_data: Rc<CommonData>
}

impl RoomViewData {
    pub fn new_from_click() -> Self {
        let cx = APP.with(|app| app.provide_scope());
        let mut accs_list = Vec::new();
        APP.with(|app| {
            app.accounts.with_untracked(|accs|
                accs_list = accs.values().cloned().collect::<Vec<Account>>()
            );
            accs_list.push(app.user.as_ref().clone());
        });
        let id = Id::new(Tb::Room);
        let msgs = cx.create_rw_signal(RoomMsgChunks::new(id.clone()));
        // let msgs_count = cx.create_memo(move |_| {
        //     msgs.with(|m| {
        //         let c = m.chunks_count;
        //         trace!("== memo(room msgs count: {c})");
        //         c
        //     })
        // });
        let owner = accs_list.remove(0);
        let _msgs_id = SignalGet::id(&msgs);
        // println!("ROOM MSGS SIGNAL ID: {msgs_id:#?}");
        Self {
            room_idx: RoomTabIdx::new(id.id),
            msgs,
            num_unread: cx.create_rw_signal(0),
            unread: cx.create_rw_signal(false),
            description: cx.create_rw_signal(None),
            owner,
            members: HashMap::from_iter(accs_list.into_iter().map(|acc | (acc.acc_id.id, acc))),
            view_id: ViewId::new(),
            room_id: id,
            last_msg: cx.create_rw_signal(None),
            common_data: APP.with(|gs| gs.common_data.clone()),
            get_update: cx.create_rw_signal(RoomMsgUpt::NoUpdate),
            msgs_count: Cell::new(0),
            is_active: cx.create_rw_signal(Cell::new(false)),
            // display_state: todo!(),
        }
    }

    /// Return room index value.
    pub fn idx(&self) -> usize {
        self.room_idx.idx
    }

    /// Update `last_msg` field on Self fetching msg from chunks using msg id.
    pub fn update_last_msg(&mut self) {
        self.msgs.with_untracked(|msgs| {
            if let Some(msg) = msgs.last_msg() {
                self.last_msg.set(Some(msg.clone()));
            } else {
                warn!("fn: update_last_msg: unable to update `last_msg` field");
            }
        })
    }

    // /// Use to apply on BTreeMap of loaded messages on room tab view.
    // pub fn get_msg_range_to_display(&self) -> Range<Ulid> {
    //     self.display_state.with_untracked(|ds| ds.get_range_to_display())
    // }
}


#[derive(Clone, Debug)]
pub struct RoomTabIdx {
    pub idx: usize,
    pub id: Ulid
}

impl RoomTabIdx {
    /// Because it's used only during room creation, new atomic index can be fetched.
    pub fn new(room_id: Ulid) -> Self {
        Self {
            idx: ROOM_IDX.fetch_add(1, Ordering::Relaxed),
            id: room_id
        }
    }

    /// Recreate full room id from Ulid.
    pub fn id(&self) -> Id {
        Id { tb: Tb::Room, id: self.id }
    }
}


impl IntoView for RoomViewData {
    type V = floem::AnyView;

    /// - [x] Selectable as a room
    /// - [x] Tracks and updates last msg status
    ///     - [ ] updates it in fine-grained way
    fn into_view(self) -> Self::V {
        let _this_room = self.room_id.id;
        let active = APP.with(|a| a.active_room);
        let last_msg = self.last_msg;
        let msgs = self.msgs;
        let get_upt = self.get_update;
        let need_avatar_change = Trigger::new();
        let need_label_change = Trigger::new();
        let need_text_change = Trigger::new();
        let need_last_msg_upt = Trigger::new();

        // -- Receive last upt from the app and evaluate if there is a need for un update
        create_effect(move |_| {
            need_last_msg_upt.track();
            debug!("== effect(room_into_view): need_last_msg_upt");
            msgs.with_untracked(|msgs| {
                if let Some(msg) = msgs.last_msg() {
                    self.last_msg.set(Some(msg.clone()));
                    batch(|| {
                        need_avatar_change.notify();
                        need_label_change.notify();
                        need_text_change.notify();
                    })
                } else {
                    warn!("fn: update_last_msg: unable to update `last_msg` field");
                }
            });
        });
        
        // -- Evaluate room event and decide if repaint is needed (TODO)
        create_effect(move |_| {
            debug!("== effect(room_view_data): msg event");
            match get_upt.get() {
                RoomMsgUpt::New => {
                    trace!("effect | room_view_data | get_update: New");
                    need_last_msg_upt.notify();
                }
                RoomMsgUpt::NewMany => {
                    trace!("effect | room_view_data | get_update: NewMany");
                    need_last_msg_upt.notify();
                },
                RoomMsgUpt::Changed(msg) => {
                    last_msg.with_untracked(|lm| {
                        if let Some(last_msg) = lm {
                            if last_msg.id.id == msg {
                                need_text_change.notify();
                            }
                        } else {
                            msgs.with_untracked(|msgs| {
                                if let Some(msg) = msgs.find_msg(msg) {
                                    // TODO: system of notification icons/marks
                                }
                            })
                        }
                    })
                },
                RoomMsgUpt::Deleted(msg) => {
                    todo!()
                }
                _ => {}
            }
        });

        // --------- views builders ---------- //

        let last_msg_text = label(move || {
            need_text_change.track();
            trace!("label: last_msg_text");
            last_msg.with_untracked(|msg| {
                if let Some(msg) = msg {
                    let text = msg.msg.text.current.clone();
                    let more_than_two_columns = text.lines().count() > 2;
                    // -- trim msg if needed
                    if more_than_two_columns {
                        let mut t: String = text.lines().take(2).collect();
                        t.push_str("...");
                        t
                    } else {
                        text
                    }
                } else { "no msgs yet..".into() }
            })
        }).style(|s| s.max_size_full().text_ellipsis());


        let last_msg_avatar = dyn_view(move || {
            need_avatar_change.track();
            trace!("dyn_view for avatar");
            img({
                move || {
                    let img_data = last_msg.with_untracked(|last_msg| {
                        if let Some(msg) = last_msg {
                            let new_av = msg.author.av.clone();
                            trace!("img author");
                            return new_av
                        } else {
                            Rc::new(Vec::with_capacity(0))
                        }
                    });
                    img_data.to_vec()
                }
            }).style(|s| s.size(50., 50.))
        }).style(|s| s
            .border(1.)
            .border_color(Color::NAVY)
            .border_radius(5.)
        );
        

        let last_msg_author = label(move || {
            need_label_change.track();
            trace!("label: last_msg_author");
            last_msg.with_untracked(|last_msg| {
                if let Some(msg) = last_msg {
                    msg.author.username.clone()
                } else {
                    String::with_capacity(0)
                }
            })
        }).style(|s| s
            .padding(5.)
            .font_bold()
            .font_size(22.)
        );
        

        let top_view = (last_msg_avatar, last_msg_author)
            .h_stack()
            .debug_name("top_room")
            .style(|s| s.gap(10.).items_center());
        let main_view = (top_view, last_msg_text)
            .v_stack()
            .debug_name("main_room")
            .style(|s| s.max_size_full().gap(10.));
        
        main_view
            .container()
            .debug_name("outer_main_room")
            .style(move |s| s
                .max_width(200.)
                .padding(2.)
                .max_height(100.)
                .border(0.5)
                .border_color(Color::NAVY)
                .apply_if(active.get().is_some_and(|r| r.id == self.room_id.id), |s| s
                    .background(Color::LIGHT_GRAY)
                    .border(2)
                    .border_color(Color::DARK_BLUE)
                )
            )
            .on_click_stop(move |_| {
                // -- If this room is not selected, select it
                match active.get_untracked() {
                    Some(id) if id.id == self.room_id.id => {
                        trace!("effect: select_room: already selected: Some({})", id.idx);
                        // APP.with(|app| app.rooms_tabs)
                    },
                    Some(_id) => {
                        trace!("effect: select_room: new room selected: {}", self.room_idx.idx);
                        active.set(Some(self.room_idx.clone()));
                        self.is_active.update(|cell| cell.set(true));
                    },
                    None => {
                        warn!("effect: select_room: fetched active_room is None, selecting current: {}", self.room_idx.idx);
                        active.set(Some(self.room_idx.clone()));
                        self.is_active.update(|cell| cell.set(true));
                        // self.is_active.set(true);
                    }
                };
            })
            .into_any()
    }
}

// MARK: Display St.

/// Track what to show up on the screen.
#[derive(Debug, Default)]
pub struct DisplayState {
    pub total_loaded_range: Range<Ulid>,
    pub what_to_load_after_back: Range<Ulid>,

}

impl DisplayState {
    /// Change whole range.
    pub fn update_loaded_range(&mut self, start: Ulid, end: Ulid) {
        self.total_loaded_range.start = start;
        self.total_loaded_range.end = end;
    }

    /// Add older messages to range (chunks?).
    pub fn add_to_start(&mut self, val: Ulid) {
        self.total_loaded_range.start = val;
    }
    
    /// Add newer messages to range.
    pub fn add_to_end(&mut self, val: Ulid) {
        self.total_loaded_range.end = val;
    }

    /// Change `what_to_load_after_back` field to sensible number(20).
    pub fn calculate_next_view_range(&self) {
        todo!()
    }

    /// Use to apply on BTreeMap of loaded messages on room tab view.
    pub fn get_range_to_display(&self) -> Range<Ulid> {
        self.what_to_load_after_back.clone()
    }
}