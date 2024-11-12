use std::rc::Rc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use chrono_lite::Datetime;
use floem::taffy::AlignItems;
use floem::{prelude::*, AnyView};

use crate::cont::acc::Account;
use crate::cont::msg::{Msg, Text};
use crate::util::{Id, Tb};

static COUNTER: AtomicU16 = AtomicU16::new(0);



#[derive(Clone, Debug, PartialEq)]
pub struct MsgCtx {
    pub id: Id,
    pub author: Rc<Account>,
    pub room: Id,
    pub room_owner: bool,
    pub com: RwSignal<Option<Vec<ComCtx>>>,
    pub rea: RwSignal<Option<Vec<ReaCtx>>>,
    pub msg: Rc<Msg>
}

impl MsgCtx {
    pub fn new_from_click(room: &Id, author: &Account) -> Self {
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
                    COUNTER.fetch_add(1, Ordering::Relaxed)
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
        Self {
            id: Id::new(Tb::Msg),
            author: Rc::new(author.clone()),
            room: room.clone(),
            com: RwSignal::new(None),
            rea: RwSignal::new(None),
            msg: Rc::new(m),
            room_owner: true
        }
    }
}

impl IntoView for MsgCtx {
    type V = AnyView;

    fn into_view(self) -> Self::V {
        let text = self.msg.text.current.clone();
        let time = self.msg.created.clone();
        let author = self.author.username.clone();
        (
            author.style(|s| s.color(Color::GRAY)),
            text,
            time.to_raw_compact().style(|s| s.color(Color::GRAY))
        )
            .v_stack()
            .debug_name("msg")
            .style(move |s| s
                // .align_self(AlignItems::FlexEnd)
                .justify_between()
                .border(1.)
                .border_color(Color::BLACK)
                .border_radius(4.)
                .height(70.)
                .min_height(70.)
                // .max_height(70.)
                // .min_width_full()
                // .max_width_full()c
                .padding(5.)
                // .apply_if(self.room_owner, |s| s.align_self(AlignItems::End))
            ).into_any()
    }
}



#[derive(Clone, Debug)]
pub struct ComCtx {
    id: Id,

}

#[derive(Clone, Debug)]
pub struct ReaCtx {
    id: Id,
}



