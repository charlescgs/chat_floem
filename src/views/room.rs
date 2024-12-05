use std::{collections::HashMap, fmt::Display, rc::Rc, sync::atomic::AtomicUsize};

use floem::{prelude::*, reactive::{create_effect, use_context, Trigger}};
use tracing_lite::{trace, warn};
use ulid::Ulid;

use crate::{cont::acc::Account, util::{Id, Tb}, ChatState, MsgView};
use super::{chunks::RoomMsgChunks, msg::MsgCtx};

pub(crate) static ROOM_IDX: AtomicUsize = AtomicUsize::new(0);



#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RoomViewUpt {
    Full,
    OnlyText,
    None
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RoomLabel {
    /// When only owner.
    None,
    /// When only one member.
    MemberName(String),
    /// When many members: group chat.
    Label(String)
}

impl Display for RoomLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoomLabel::None => f.write_str(""),
            RoomLabel::MemberName(mn) => f.write_str(mn),
            RoomLabel::Label(l) => f.write_str(l)
        }
    }
}


/// main structure for the room essential data.
#[derive(Clone, Debug, PartialEq)]
pub struct RoomCtx {
    /// Room Id.
    pub id: Id,
    /// Room label being shown for the user.  
    /// Can be member/owner name, custom or None,.
    pub label: RwSignal<RoomLabel>,
    /// Avatar for the last msg view.
    pub last_msg_avatar: RwSignal<Option<Vec<u8>>>,
    /// Owner data.
    pub owner: Account,
    /// All room members (without the owner).
    pub members: HashMap<Ulid, Account>,
    /// All room messages grouped in `Chunks`.
    pub msgs: RwSignal<RoomMsgChunks>,
    /// Last Msg (if any).
    pub last: RwSignal<Option<MsgCtx>>,
    /// If contains any unread messages.
    pub unread: RwSignal<bool>,
    /// Number of unread messages.
    pub num_unread: RwSignal<u16>,
    /// Optional room description.
    pub description: RwSignal<Option<String>>
}

impl RoomCtx {
    pub fn new_from_click(st: Rc<ChatState>) -> Self {
        let acc = if let Some(acc) = Account::new_from_click() {
            acc
        } else {
            st.accounts.with_untracked(|accs| accs.values().next().unwrap().clone())
        };
        let id = Id::new(Tb::Room);
        Self {
            msgs: RwSignal::new(RoomMsgChunks::new(id.clone())),
            id,
            label: RwSignal::new(RoomLabel::None),
            num_unread: RwSignal::new(0),
            unread: RwSignal::new(false),
            description: RwSignal::new(None),
            owner: acc,
            members: HashMap::new(),
            last: RwSignal::new(None),
            last_msg_avatar: RwSignal::new(None),
        }
    }
}


impl IntoView for RoomCtx {
    type V = floem::AnyView;

    fn into_view(self) -> Self::V {
        let state = use_context::<Rc<ChatState>>().unwrap();
        let rooms_msgs = state.rooms_msgs;
        let active_room = state.active_room;

        let msg_view = use_context::<RwSignal<MsgView>>().unwrap();
        // let new_msg_scroll_end = use_context::<Trigger>().unwrap();
        let room_selected = Trigger::new(); // TODO: have saved last room_id to minimize checks
        // -- Triggers for fine-grained updates
        let _need_update = RwSignal::new(RoomViewUpt::None);
        let need_avatar_change = Trigger::new();
        let need_label_change = Trigger::new();
        let need_text_change = Trigger::new();

        let room = Rc::new(self);
        let room_id = room.id.id;
        let last_msg = room.last;

        // -- Effect to evaulate if last msg changed and therefore is a need to update room_view
        // TODO: make it update only text and fetch avatar and name only if person changed
        create_effect(move |_| {
            trace!("effect: evaluate `last_msg` for {room_id}");
            if let MsgView::NewMsg(id) = msg_view.get() {
                if id.id == room_id {
                    trace!("{room_id} needs update");
                    
                }
            }
        });


        let last_msg_text = label(move || {
            need_text_change.track();
            trace!("label: last_msg_text");
            rooms_msgs.with_untracked(|rooms| {
                if let Some(msgs) = rooms.get(&room_id) {
                    if let Some(msg_ctx) = msgs.with_untracked(|r| r.last_msg().cloned()) {
                        let text = msg_ctx.msg.text.current.clone();
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
        
        // -- Select active room
        create_effect(move |_| {
            room_selected.track();
            trace!("effect: select_room");
            let act_room_state = active_room.get_untracked();
            match act_room_state {
                Some(id) if id.id == room_id => {
                    trace!("effect: select_room: is Some({id})");
                    false
                },
                Some(id) => {
                    trace!("effect: select_room: new room selected: {id}");
                    active_room.set(Some(room.id.clone()));
                    // msg_view.set(MsgView::NewMsg(room.id.clone()));
                    false
                },
                None => {
                    warn!("effect: select_room fetched active_room is None, selecting current..");
                    active_room.set(Some(room.id.clone()));
                    true
                }
            }
            // if need_upt {
                // trace!("effect: {} needs_upt", room.id);
                // new_msg_scroll_end.notify();
            // }
        });

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
            .style(|s| s
                .max_width(200.)
                .padding(2.)
                .max_height(100.)
                .border(0.5)
                .border_color(Color::NAVY))
            .into_any()
            .on_click_stop(move |_| {
                room_selected.notify();
            })
    }
}