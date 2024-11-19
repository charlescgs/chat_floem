use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use chrono_lite::Datetime;
use floem::style::TextOverflow;
use floem::taffy::AlignItems;
use floem::{prelude::*, AnyView};
use ulid::Ulid;

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
    pub fn new(msg: Msg, author: &Account, owner: bool) -> Self {
        Self {
            id: msg.msg_id.clone(),
            author: Rc::new(author.clone()),
            room: msg.room_id.clone(),
            com: RwSignal::new(None),
            rea: RwSignal::new(None),
            msg: Rc::new(msg),
            room_owner: owner
        }
    }
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
            text
                .style(|s| s
                    // .border(1.)
                    // .text_overflow(TextOverflow::Wrap)
                    // .border_color(Color::BLACK)
                    // .flex_grow(1.)
                    // .max_height_full()
                ),
            time.human_formatted().style(|s| s.color(Color::GRAY))
        )
            .v_stack()
            .debug_name("msg")
            .style(move |s| s
                // .justify_between()
                .border(1.)
                .border_color(Color::BLACK)
                .border_radius(5.)
                .padding(5.)
                // .text_overflow(TextOverflow::Wrap)
                // .max_height_full()
                // .max_width_full()
                .min_height(50.)
                // .max_width_pct(80.)
                // .flex_basis(70.)
                // .flex_grow(0.)
                // .flex_shrink(1.)
            )
            .into_any()
    }
}

pub fn layout_text(text: String) -> Label {
    todo!()
}



#[derive(Clone, Debug)]
pub struct ComCtx {
    id: Id,

}

#[derive(Clone, Debug)]
pub struct ReaCtx {
    id: Id,
}



// MARK: Chunks


/// Struct holding info regarding msgs for the room.
/// 
/// 
#[derive(Debug, Clone, PartialEq)]
pub struct RoomMsgChunks {
    /// Total room msgs.
    pub total_msgs: u16,
    /// Total room msgs.
    pub chunks_count: u8,
    /// Msgs as chunks.
    pub chunks: Vec<Rc<MsgChunk>>,
    /// Displayed chunk index.
    pub chunk_on_display: u8,
    // pub from: Ulid,
    // pub to: Ulid
}


#[derive(Debug, Clone, PartialEq)]
pub struct MsgChunk {
    /// Max: 20 (for now).
    pub count: u8,
    pub msgs: BTreeMap<Ulid, MsgCtx>
}


#[derive(Debug, Clone, Default, PartialEq)]
pub struct LoadedRange {
    
}