use std::{collections::HashMap, fmt::Display, rc::Rc};

use floem::{prelude::*, reactive::{create_effect, use_context, Trigger}, AnyView, IntoView};
use tracing_lite::trace;
use ulid::Ulid;

use crate::{cont::acc::Account, util::{Id, Tb}, ChatState};
use super::msg::MsgCtx;



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


#[derive(Clone, Debug, PartialEq)]
pub struct RoomCtx {
    pub id: Id,
    pub label: RwSignal<RoomLabel>,
    pub owner: Account,
    pub members: HashMap<Ulid, Account>,
    // pub msgs: RwSignal<BTreeMap<Ulid, MsgCtx>>,
    pub last: RwSignal<Option<MsgCtx>>,
    pub unread: RwSignal<bool>,
    pub num_unread: RwSignal<u16>,
    pub description: RwSignal<Option<String>>,
}

impl RoomCtx {
    pub fn new_from_click(st: Rc<ChatState>) -> Self {
        // let room_id = Id::new(Tb::Room);
        let acc = if let Some(acc) = Account::new_from_click() {
            acc
        } else {
            st.accounts.with_untracked(|accs| accs.values().next().unwrap().clone())
        };
        Self {
            id: Id::new(Tb::Room),
            label: RwSignal::new(RoomLabel::None),
            num_unread: RwSignal::new(0),
            unread: RwSignal::new(false),
            description: RwSignal::new(None),
            owner: acc,
            members: HashMap::new(),
            last: RwSignal::new(None)
        }
    }
}


impl IntoView for RoomCtx {
    type V = AnyView;

    fn into_view(self) -> Self::V {
        let state = use_context::<Rc<ChatState>>().unwrap();
        // let msgs_tracker = use_context::<Trigger>().unwrap();
        let msgs_trackerv2 = use_context::<RwSignal<Option<Id>>>().unwrap();
        let selected = Trigger::new();
        let need_update = Trigger::new();

        let state2 = state.clone();
        let state3 = state.clone();
        let state5 = state.clone();
        let room = Rc::new(self);
        let room2 = room.clone();
        let room3 = room.clone();
        let room4 = room.clone();
        let room5 = room.clone();
        let room6 = room.clone();

        // -- Effect to evaulate if last msg changed and therefore is a need to update room_view
        create_effect(move |_| {
            trace!("Evaluate 'last msg' for {}", room6.id);
            if let Some(id) = msgs_trackerv2.get() {
                if id == room6.id {
                    trace!("{} needs update", room6.id);
                    need_update.notify()
                }
            }
        });
        // let last_msg_memo = create_memo(move |_| {
        //     // msgs_tracker.track();
        //     trace!("->> last_msg_memo");
        //     state4.data.with_untracked(|data| {
        //         if let Some(msgs) = data.get(&self.id.id) {
        //             if let Some((id, msg_ctx)) = msgs.borrow().last_key_value() {
        //                 trace!("->> last_msg_memo: got {} {}", msg_ctx.author.username, msg_ctx.msg.text.current);
        //                 (
        //                     msg_ctx.author.av.clone(),
        //                     msg_ctx.author.username.clone(),
        //                     msg_ctx.msg.text.current.clone()
        //                 )
        //             } else { ("---".into(), "---".into(), "no msg yet".into()) }
        //         } else { ("--".into(), "--".into(), "no msg yet".into()) }
        //     })
        // });

        let av = dyn_view(move || {
            need_update.track();
            trace!("dyn_view for avatar");
            img({
                let st = state5.clone();
                let room = room2.clone();
                move || {
                    let img_data = st.data.with_untracked(|rooms| {
                        if let Some(msgs) = rooms.get(&room.id.id) {
                            if let Some((_, msg_ctx)) = msgs.borrow().last_key_value() {
                                if room.owner.acc_id == msg_ctx.msg.author {
                                    trace!("img author");
                                    room.owner.av.clone()
                                } else {
                                    trace!("img member");
                                    match room.members.get(&msg_ctx.msg.author.id) {
                                        Some(acc) => acc.av.clone(),
                                        None => Rc::new(Vec::with_capacity(0))
                                    }
                                }
                            } else { Rc::new(Vec::with_capacity(0)) }
                        } else { Rc::new(Vec::with_capacity(0)) }
                    });
                    img_data.to_vec()
                }
            }).style(|s| s.size(50., 50.))
        })
        .style(|s| s
            .border(1.)
            .border_color(Color::NAVY)
            .border_radius(5.)
        );
        
        let author = label(move || {
            need_update.track();
            state.data.with_untracked(|data| {
                trace!("author");
                if let Some(msgs) = data.get(&room3.id.id) {
                    if let Some((_, msg_ctx)) = msgs.borrow().last_key_value() {
                        msg_ctx.author.username.clone()
                    } else { "".into() }
                } else { "".into() }
            })
        })
            .style(|s| s
                .padding(5.)
                .font_bold()
                .font_size(22.)
            );
        
        let last_msg = label(move || {
            need_update.track();
            // let (av, us, tx) = last_msg_memo.get_untracked();
            // (img(move || av.clone()),
            // label(move || us.clone()),
            // label(move || tx.clone()))
            state2.data.with_untracked(|rooms| {
                trace!("current text");
                if let Some(msgs) = rooms.get(&room4.id.id) {
                    if let Some((_, msg_ctx)) = msgs.borrow().last_key_value() {
                        msg_ctx.msg.text.current.clone()
                    } else { "no msgs yet..".into() }
                } else { "no msgs yet..".into() }
            })
        });
        

        // let room_id = room5.id.clone();
        create_effect(move |_| {
            selected.track();
            trace!("effect: 'select room'");
            let need_upt = state3.active.with_untracked(|act| {
                match act {
                    Some(id) if id == &room5.id => false,
                    _ => true
                }
            });
            if need_upt {
                trace!("into_view for RoomCtx: need_upt is `true`");
                state3.active.set(Some(room.id.clone()));
                msgs_trackerv2.set(Some(room.id.clone()));
                // msgs_tracker.notify();
            }
        });

        
        // let us = move || last_msg_memo.with_untracked(|lm| lm.1.clone());
        // let text = move || last_msg_memo.with_untracked(|lm| lm.2.clone());
        // last_msg()
        //     .h_stack()
        //     .style(|s| s.gap(10.).items_center())
        let top_view = (av, author)
            .h_stack()
            .debug_name("top_room")
            .style(|s| s.gap(10.).items_center());
        let main_view = (top_view, last_msg)
            .v_stack()
            .debug_name("main_room")
            .style(|s| s.gap(10.));
        
        main_view
            .container()
            .debug_name("outer_main_room")
            .style(|s| s
                .width_full()
                .padding(2.)
                .height(100.)
                .border(0.5)
                .border_color(Color::NAVY))
            .into_any()
            .on_click_stop(move |_| {
                selected.notify();
            })
    }
}