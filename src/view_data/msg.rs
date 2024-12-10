use std::rc::Rc;
use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering;
use std::time::Duration;

use chrono_lite::Datetime;
use floem::prelude::*;
use floem::ViewId;
use im::{vector, Vector};
use tracing_lite::info;

use crate::cont::msg::Text;
use crate::util::{Id, Tb};
use crate::views::msg::{ComCtx, ReaCtx};
use crate::common::CommonData;
use crate::cont::acc::Account;
use crate::cont::msg::Msg;
use super::session::APP;


static MSG_VIEW_COUNTER: AtomicU16 = AtomicU16::new(0);


/// Contains data needed to display msg widget on the msgs list.
#[derive(Clone, Debug)]
pub struct MsgViewData {
    pub view_id: ViewId,
    pub id: Id,
    pub author: Rc<Account>,
    pub room: Id,
    pub room_owner: bool,
    pub msg: Rc<Msg>,
    pub com: RwSignal<Vector<ComCtx>>,
    pub rea: RwSignal<Vector<ReaCtx>>,
    pub common_data: Rc<CommonData>
}

impl MsgViewData {
    pub fn new(msg: Msg, author: &Account, owner: bool) -> Self {
        let cx = APP.with(|app| app.provide_scope());
        Self {
            author: Rc::new(author.clone()),
            com: cx.create_rw_signal(vector!()),
            rea: cx.create_rw_signal(vector!()),
            room: msg.room_id.clone(),
            msg: Rc::new(msg),
            room_owner: owner,
            view_id: ViewId::new(),
            id: Id::new(Tb::Msg),
            common_data: APP.with(|gs| gs.common_data.clone())
        }
    }
    
    pub fn new_from_click(room: Id, author: &Account) -> Self {
        let msg_id = Id::new(Tb::Msg);
        let m = Msg {
            msg_id: msg_id.clone(),
            room_id: room.clone(),
            author: author.acc_id.clone(),
            created: Datetime::default().sub_from(Duration::from_secs(5)),
            sent: Some(Datetime::default()),
            text: Text {
                current: String::from(format!(
                    "Really important message no: {}",
                    MSG_VIEW_COUNTER.fetch_add(1, Ordering::Relaxed)
                )),
                edits: None,
                last_edited: None
            },
            media: None,
            edited: None,
            comments: None,
            reactions: None,
            delivered_to_all: true,
            viewed_by_all: true,
        };
        let cx = APP.with(|app| app.provide_scope());
        Self {
            id: Id::new(Tb::Msg),
            author: Rc::new(author.clone()),
            room: room.clone(),
            com: cx.create_rw_signal(vector!()),
            rea: cx.create_rw_signal(vector!()),
            msg: Rc::new(m),
            room_owner: true,
            view_id: ViewId::new(),
            common_data: APP.with(|gs| gs.common_data.clone())
        }
    }
}


impl IntoView for MsgViewData {
    type V = floem::AnyView;
    
    fn into_view(self) -> Self::V {
        info!("->> into_view(msg) | {}", self.id.id);
        let text = self.msg.text.current.clone();
        let time = self.msg.created.clone();
        let author = self.author.username.clone();
        (
            author.style(|s| s.color(Color::GRAY)),
            text,
            time.human_formatted().style(|s| s.color(Color::GRAY))
        )
            .v_stack()
            .debug_name("msg")
            .style(move |s| s
                .justify_between()
                .border(1.)
                .border_color(Color::BLACK)
                .border_radius(5.)
                .padding(5.)
                .min_height(40.)
                // .min_width_pct(20.)
                .max_width_pct(80.)
            )
            .into_any()
    }
}

impl PartialOrd for MsgViewData {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.id.cmp(&other.id.id))
    }
}

impl Ord for MsgViewData {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.id.cmp(&other.id.id)
    }
}

impl PartialEq for MsgViewData {
    fn eq(&self, other: &Self) -> bool {
        self.id.id == other.id.id
    }
}

impl Eq for MsgViewData {}