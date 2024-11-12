use std::{collections::{BTreeMap, HashMap, HashSet}, fmt::Display, rc::Rc};

use floem::{prelude::*, reactive::{create_effect, create_memo, use_context, Trigger}, text::Style, AnyView, IntoView};
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
    // pub last: RwSignal<Option<MsgCtx>>,
    pub unread: RwSignal<bool>,
    pub num_unread: RwSignal<u16>,
    pub description: RwSignal<Option<String>>,
}

impl RoomCtx {
    pub fn new_from_click(st: Rc<ChatState>) -> Self {
        let room_id = Id::new(Tb::Room);
        let acc = if let Some(acc) = Account::new_from_click() {
            acc
        } else {
            st.accounts.with_untracked(|accs| accs.values().next().unwrap().clone())
        };
        // let msg = MsgCtx::new_from_click(&room_id, &author);
        // let msg2 = MsgCtx::new_from_click(&room_id, &author);
        // let msg3 = MsgCtx::new_from_click(&room_id, &author);
        Self {
            id: Id::new(Tb::Room),
            label: RwSignal::new(RoomLabel::None),
            // msgs: RwSignal::new(BTreeMap::from([
            //     (msg.id.id.clone(), msg),
            //     (msg2.id.id.clone(), msg2),
            //     (msg3.id.id.clone(), msg3)
            // ])),
            // msgs: RwSignal::new(BTreeMap::new()),
            num_unread: RwSignal::new(0),
            unread: RwSignal::new(false),
            // last: RwSignal::new(msg.clone()),
            description: RwSignal::new(None),
            owner: acc,
            members: HashMap::new()
        }
    }
}


impl IntoView for RoomCtx {
    type V = AnyView;

    fn into_view(self) -> Self::V {
        let state = use_context::<Rc<ChatState>>().unwrap();
        let state2 = state.clone();

        let (avatar, name) = state.data.with(|data| {
            if let Some(msgs) = data.get(&self.id.id) {
                if let Some((id, msg_ctx)) = msgs.borrow().last_key_value() {
                    (msg_ctx.author.av.clone(), msg_ctx.author.username.clone())
                } else { ("---".into(), "---".into()) }
            } else { ("--".into(), "--".into()) }
        });
        let av = img(move || avatar.clone())
            .style(|s| s
                .justify_center()
                .items_center()
                .size_full()
                .max_size_full()
                // .border(1.)
                // .border_color(Color::NAVY)
                .border_radius(5.)
            );
        let room_name = label(move || { name.clone()
            // state.data.with(|data| {
            //     trace!("last msg..");
            //     if let Some(msgs) = data.get(&self.id.id) {
            //         if let Some((id, msg_ctx)) = msgs.borrow().last_key_value() {
            //             msg_ctx.m
            //         } else { "no msgs yet..".into() }
            //     } else { "no msgs yet..".into() }
            // })
        })
            .style(|s| s
                .padding(10.)
                .font_bold()
                .font_size(24.)
            );
        let last_msg = label(move || {
            state.data.with(|rooms| {
                trace!("last msg..");
                if let Some(msgs) = rooms.get(&self.id.id) {
                    if let Some((id, msg_ctx)) = msgs.borrow().last_key_value() {
                        msg_ctx.msg.text.current.clone()
                    } else { "no msgs yet..".into() }
                } else { "no msgs yet..".into() }
            })
        });
        let selected = Trigger::new();
        let msgs_tracker = use_context::<Trigger>().unwrap();
        let room = self.clone();

        let room_id = self.id.clone();
        create_effect(move |_| {
            selected.track();
            trace!("effect: 'select room'");
            let need_upt = state2.active.with_untracked(|act| {
                match act {
                    Some(id) if id == &self.id => false,
                    _ => true
                }
            });
            if need_upt {
                trace!("into_view for RoomCtx: need_upt is `true`");
                state2.active.set(Some(room.id.clone()));
                msgs_tracker.notify();
            }
        });
        
        let top_view = (av, room_name)
            .h_stack()
            .style(|s| s.gap(10.).items_center());
        let main_view = (top_view, last_msg)
            .v_stack()
            .style(|s| s.gap(10.));
        
        main_view
            .container()
            .style(|s| s
                .width_full()
                .height(100.)
                .border(1.)
                .border_color(Color::NAVY))
            .into_any()
            .on_click_stop(move |_| {
                selected.notify();
            })
    }
}