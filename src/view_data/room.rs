use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::Ordering;

use floem::reactive::{batch, create_effect, use_context, Trigger};
use floem::{prelude::*, ViewId};
use tracing_lite::{debug, trace, warn};
use ulid::Ulid;

use crate::cont::msg::MsgEdit;
use crate::util::Tb;
use crate::views::chunks::RoomMsgChunks;
use crate::common::CommonData;
use crate::cont::acc::Account;
use crate::views::msg::MsgCtx;
use crate::util::Id;
use crate::views::room::ROOM_IDX;

use super::session::APP;
use super::MsgEvent;


#[derive(Clone)]
/// Main structure containing all data needed for room view and msgs view.
pub struct RoomViewData {
    pub view_id: ViewId,
    pub room_id: Id,
    pub room_idx: RoomTabIdx,
    // pub is_selected: RwSignal<bool>,
    pub owner: Account,
    pub members: HashMap<Ulid, Account>,
    pub last_msg: RwSignal<Option<MsgCtx>>,
    pub msgs: RwSignal<RoomMsgChunks>,
    pub unread: RwSignal<bool>,
    pub num_unread: RwSignal<u16>,
    pub description: RwSignal<Option<String>>,
    pub common_data: Rc<CommonData>
}

impl RoomViewData {
    pub fn new_from_click() -> Self {
        let acc = if let Some(acc) = Account::new_from_click() {
            acc
        } else {
            APP.with(|gs| gs.accounts.with_untracked(|accs|
                accs.values().next().unwrap().clone()
            ))
        };
        let id = Id::new(Tb::Room);
        Self {
            room_idx: RoomTabIdx::new(id.id),
            msgs: RwSignal::new(RoomMsgChunks::new(id.clone())),
            num_unread: RwSignal::new(0),
            unread: RwSignal::new(false),
            description: RwSignal::new(None),
            owner: acc,
            members: HashMap::new(),
            view_id: ViewId::new(),
            room_id: id,
            last_msg: RwSignal::new(None),
            common_data: APP.with(|gs| gs.common_data.clone()),
            // is_selected: RwSignal::new(false)
        }
    }

    /// Return room index value.
    pub fn idx(&self) -> usize {
        self.room_idx.idx
    }
}


#[derive(Clone, Debug)]
pub struct RoomTabIdx {
    pub idx: usize,
    pub id: Ulid
}

impl RoomTabIdx {
    /// Because it's used only during room creation, new atomic index can be fetched.
    pub fn new(room_id: Ulid) -> Self {
        // let idx = APP.with(|gs|
        //     gs.rooms_tabs.with_untracked(|tabs|
        //         tabs.get(&room_id).cloned().unwrap_or_default(0)
        //     ).0
        // );
        Self {
            idx: ROOM_IDX.fetch_add(1, Ordering::Relaxed),
            id: room_id
        }
    }
}


impl IntoView for RoomViewData {
    type V = floem::AnyView;

    /// - [ ] Selectable as a room
    /// - [ ] Tracks and updates last msg status
    ///     - [ ] updates it in fine-grained way
    fn into_view(self) -> Self::V {
        let this_room = self.room_id.id;
        let active = APP.with(|a| a.active_room);
        let is_selected = RwSignal::new(false);
        let last_msg = self.last_msg;
        let _msgs = self.msgs;

        let need_avatar_change = Trigger::new();
        let need_label_change = Trigger::new();
        let need_text_change = Trigger::new();
        // -- Receive last upt from the app and evaluate if there is a need for un update
        let msg_update = use_context::<RwSignal<MsgEvent>>().unwrap();

        // // -- De-select if active room changed
        // create_effect(move |_| {
        //     debug!("effect: de-select if active room changed");
        //     match active.get() {
        //         Some(act) => {
        //             if act.id != this_room {
        //                 // is_selected.set(false);
        //             }
        //         },
        //         None => {}
        //     }
        // });

        // -- last msg effect
        create_effect(move |_| {
            match msg_update.get() {
                MsgEvent::NewFor(r) if r == this_room => {
                    batch(|| {
                        need_avatar_change.notify();
                        need_label_change.notify();
                        need_text_change.notify();
                    });
                },
                MsgEvent::UpdatedFor { msg, room } if room == this_room => {
                    last_msg.with_untracked(|lm| {
                        if let Some(last_msg) = lm {
                            if last_msg.id.id == msg {
                                need_text_change.notify();
                            }
                        }
                    })
                },
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
            .style(move |s| s // needs to be 'move', right?
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
                // active.set(Some(RoomTabIdx::new(this_room)));
                // is_selected.set(true);
                let need_upt = match active.get_untracked() {
                    Some(id) if id.id == self.room_id.id => {
                        trace!("effect: select_room: is Some({})", id.id);
                        // false
                    },
                    Some(id) => {
                        trace!("effect: select_room: new room selected: {}", id.id);
                        active.set(Some(self.room_idx.clone()));
                        // msg_view.set(MsgView::NewMsg(room.id.clone()));
                        // is_selected.set(false);
                    },
                    None => {
                        warn!("effect: select_room fetched active_room is None, selecting current..");
                        active.set(Some(self.room_idx.clone()));
                        // is_selected.set(true);
                    }
                };
                // if need_upt {
                //     // TODO: notify msgs
                //     is_selected.set(true);
                // }
            })
            .into_any()
    }
}