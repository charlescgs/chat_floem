use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use chrono_lite::Datetime;
use floem::style::TextOverflow;
use floem::taffy::AlignItems;
use floem::{prelude::*, AnyView};
use im_rc::{vector, Vector};
use tracing_lite::{debug, error, info, trace, warn};
use ulid::Ulid;

use crate::cont::acc::Account;
use crate::cont::msg::{Msg, Text};
use crate::element;
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
        trace!("MsgCtx into_view()");
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
/// ### How it works:
/// ```md
/// |------------------| <- BTreeMap with Msgs
/// |------| |----| |--| <- Chunks are created
///    1       2     3   <- ..and stored in `RoomMsgChunks`
/// 
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RoomMsgChunks {
    /// Total room msgs count.
    pub total_msgs: u16,
    /// Total room chunks count.
    pub chunks_count: u16,
    /// Msgs as chunks (Oldest in front).
    pub chunks: Vec<Rc<MsgChunk>>,
    /// Index of a last displayed [MsgChunk] in `chunks`.  
    /// When loading more msg, index goes down.
    pub last_chunk_on_display: u16
    // pub from: Ulid,
    // pub to: Ulid
}

impl RoomMsgChunks {
    /// Create new chunks from message map.
    pub fn new(msgs: BTreeMap<Ulid, MsgCtx>) -> Self {
        let total_msgs = msgs.len() as u16;
        info!("total msgs: {total_msgs}");
        let chunks = {
            match total_msgs {
                1..=20 => {
                    trace!("1 to 20 msgs");
                    let mut chunk = vector!();
                    for each in msgs.into_values() {
                        chunk.push_front(each);
                    }
                    vec![Rc::new(MsgChunk::new(chunk))]
                },
                21.. => {
                    trace!("more than 21 msgs");
                    let mut chunks = Vec::new();
                    let loops = msgs.len() / 20;
                    let mut iter_on_values = msgs.values();
                    for n in 0..loops {
                        trace!("loop for {n} in {loops}");
                        let mut chunk = vector!();
                        for n in 0..20 {
                            trace!("chunks for {n} in 20");
                            if let Some(element) = iter_on_values.next() {
                                warn!("loop msg: {}", element.id.id);
                                chunk.push_front(element.clone());
                            } else {
                                trace!("loop break");
                                break
                            }
                        }
                        chunks.push(Rc::new(MsgChunk::new(chunk)));
                    }
                    chunks
                }
                _ => {
                    trace!("0 msgs");
                    vec!()
                }
            }
        };
        let chunks_count = if chunks.is_empty() { 0 } else {
            chunks.len() as u16
        };
        info!("chunks_count: {chunks_count}");
        let last_chunk_on_display = if chunks_count == 0 { 0 } else {
            chunks_count - 1
        };
        info!("last_chunk_on_display: {last_chunk_on_display}");
        Self {
            total_msgs,
            last_chunk_on_display,
            chunks_count,
            chunks
        }
    }

    pub fn load_next_chunk(&mut self) -> Rc<MsgChunk> {
        debug!("load_next_chunk");
        // -- Check how many chunks is loaded and return if no more left
        // if self.last_chunk_on_display == 0 {
        //     trace!("nothing to load");
        //     return Rc::new(MsgChunk::default())
        // }
        // -- Load another one (if exist)
        if let Some(next) = self.chunks.get(self.last_chunk_on_display as usize) {
            trace!("loading next..");
            let n = next.clone();
            self.last_chunk_on_display.saturating_sub(1);
            return n
        }
        Rc::new(MsgChunk::default())
    }
}


#[derive(Debug, Clone, Default, PartialEq)]
pub struct MsgChunk {
    /// Max msgs per chunk: 20 (for now).
    pub count: u8,
    pub msgs: Vector<MsgCtx>
}

impl MsgChunk {
    pub fn new(msgs: Vector<MsgCtx>) -> Self {
        Self {
            count: msgs.len() as u8,
            msgs
        }
    }
}