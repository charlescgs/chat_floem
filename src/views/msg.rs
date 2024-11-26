use std::cell::Cell;
use std::collections::BTreeMap;
use std::ops::Add;
use std::rc::Rc;
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use chrono_lite::Datetime;
use floem::{prelude::*, AnyView};
use im_rc::{vector, Vector};
use tracing_lite::{debug, error, info, trace, warn};
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
/// Then chunks are loaded into msg view fn as Single struct calling a method
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RoomMsgChunks {
    pub room_id: Id,
    /// Total room msgs count.
    pub total_msgs: u16,
    /// Total room chunks count.
    pub chunks_count: u16,
    /// Msgs as chunks (Oldest in front).
    pub chunks: Vec<Rc<MsgChunk>>,
    /// Index of a last displayed [MsgChunk] in `chunks`.  
    /// When loading more msg, index goes up.
    pub last_chunk_on_display: Cell<u16>
    // pub from: Ulid,
    // pub to: Ulid
}

impl RoomMsgChunks {
    pub fn new_from_single_msg(msg: MsgCtx) -> Self {
        let mut chunk = MsgChunk::default();
        let room_id = msg.room.clone();
        chunk.add_msg(msg);
        Self {
            room_id,
            total_msgs: 1,
            chunks_count: 1,
            chunks: vec!(Rc::new(chunk)),
            last_chunk_on_display: Cell::new(0)
        }
    }
    /// Create new chunks from message map.
    pub fn new(msgs: BTreeMap<Ulid, MsgCtx>, room_id: Id) -> Self {
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
        Self {
            room_id,
            total_msgs,
            last_chunk_on_display: Cell::new(chunks_count),
            chunks_count,
            chunks,
        }
    }

    /// Add single [Msg] to the chunk.
    pub fn append_new_msg(&mut self, msg: MsgCtx) {
        // Get the chunk with youngest msgs and check if full
        match self.chunks.last_mut() {
            Some(chunk) => {
                if chunk.count >= 20 {
                    // -- Create new chunk
                    self.chunks.push(Rc::new(MsgChunk::new(Vector::unit(msg))));
                    self.chunks_count += 1;
                    self.total_msgs += 1;

                    let mut display_idx = self.last_chunk_on_display.get();
                    self.last_chunk_on_display.set(display_idx.saturating_add(1));
                } else {
                    // -- Push onto existing chunk
                    if let Some(mut_chunk) = Rc::get_mut(chunk) {
                        mut_chunk.add_msg(msg);
                        self.total_msgs += 1;
                    } else {
                        error!("Rc::get_mut on Chunk returned None")
                    }
                }
            },
            None => {
                self.chunks.push(Rc::new(MsgChunk::new(Vector::unit(msg))));
                self.chunks_count += 1;
                self.total_msgs += 1;
                self.last_chunk_on_display.set(0)
            },
        }
    }

    /// Evaluate if any more not loaded chunks left in the room.
    /// TODO: check it!
    pub fn anymore_available(&self) -> bool {
        self.chunks_count > self.last_chunk_on_display.get()
    }

    #[deprecated]
    pub fn load_next_chunk(&self) -> Rc<MsgChunk> {
        debug!("load_next_chunk");
        // -- Check how many chunks is loaded and return if no more left
        if self.chunks_count == 0 {
            trace!("nothing to load");
            return Rc::new(MsgChunk::default())
        }
        // -- Load another one (if exist)
        if let Some(next) = self.chunks.get(self.last_chunk_on_display.get() as usize) {
            let old = self.last_chunk_on_display.get();
            self.last_chunk_on_display.set(old.saturating_sub(1));
            trace!("loading next..");
            let chunk = next.clone();
            return chunk
        }
        Rc::new(MsgChunk::default())
    }
    
    /// Load room chunks in range until `last_chunk_on_display` + one.
    pub fn load_next(&self) -> &[Rc<MsgChunk>] {
        debug!("fn: load_next");
        // -- Check how many chunks is loaded and return if no more left
        if self.chunks_count == 0 {
            trace!("nothing to load");
            return [].as_slice()
        }
        // -- Load another one
        trace!("self.last_chunk_on_display: {}", self.last_chunk_on_display.get());
        let mut old = self.last_chunk_on_display.get();
        let range = {
            if old == self.chunks_count.saturating_sub(1) {
                let range = &self.chunks[old as usize..];
                self.last_chunk_on_display.set(old.saturating_sub(1));
                range
            } else {
                self.last_chunk_on_display.set(old.saturating_sub(1));
                let range = &self.chunks[old as usize..];
                range
            }
        };
        range
    }
    
    /// Reload loaded chunks (as a result of new/changed message).
    pub fn reload(&self) -> &[Rc<MsgChunk>] {
        debug!("fn: reload");
        // -- Check how many chunks is loaded and return if no more left
        if self.chunks_count == 0 {
            trace!("nothing to reload");
            return [].as_slice()
        }
        // -- Change slice range to 1 if display was 0
        let mut display_idx = self.last_chunk_on_display.get();
        trace!("self.last_chunk_on_display: {display_idx}");

        let reloaded = {
            if display_idx == self.chunks_count.saturating_sub(1) {
                &self.chunks[display_idx as usize..]
            } else {
                &self.chunks[(display_idx + 1) as usize..]
            }
        };
        trace!("reloading..");
        reloaded
    }
}


#[derive(Debug, Clone, Default, PartialEq)]
pub struct MsgChunk {
    /// Max msgs per chunk: 20 (for now).
    pub count: u8,
    pub msgs: Vector<MsgCtx>
}

impl MsgChunk {
    // Construct new MsgChunk from the Vector<MsgCtx>.
    pub fn new(msgs: Vector<MsgCtx>) -> Self {
        Self {
            count: msgs.len() as u8,
            msgs
        }
    }

    /// Add single [MsgCtx] onto back of the [Vector].
    pub fn add_msg(&mut self, msg: MsgCtx) {
        self.msgs.push_back(msg);
        self.count += 1
    }
}


#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::time::Duration;
    use crate::cont::acc::Account;
    use crate::util::{Id, Tb};
    use super::{MsgCtx, RoomMsgChunks};

    #[test]
    fn basic_chunk_test() {
        // 1. Create test msgs
        let act_room = Id::new(Tb::Room);
        let acc = Account {
            acc_id: Id::new(Tb::Acc),
            username: "Karol".into(),
            av: Rc::new(vec![]),
        };

        let mut msgs_vec = Vec::with_capacity(80);
        for _ in 0..80 {
            std::thread::sleep(Duration::from_millis(2));
            let msg = MsgCtx::new_from_click(&act_room, &acc);
            msgs_vec.push(msg);
        }
        let len = msgs_vec.len();
        // 2. Insert them into RoomMsgChunks
        let mut room_chunks = RoomMsgChunks::default();
        room_chunks.room_id = act_room.clone();
        for each in msgs_vec {
            room_chunks.append_new_msg(each);
        }
        // 3. Print it and assert.
        println!("->> Before:");
        println!("room_id: {}", room_chunks.room_id);
        println!("anymore_available: {}", room_chunks.anymore_available());
        println!("msgs_vec len: {len}");
        println!("chunks_count: {}", room_chunks.chunks_count);
        println!("last_chunk_on_display: {}", room_chunks.last_chunk_on_display.get());
        println!("total_msgs: {}", room_chunks.total_msgs);

        let reload = room_chunks.reload();
        assert!(reload.iter().fold(0, |mut count, m| {count += m.count; count}) == 20);
        assert!(reload.len() == 1);
        // println!("->> reload:");
        // println!("count: {}", );
        // println!("msgs: {:#?}", reload
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );
        let load_next = room_chunks.load_next();
        assert!(load_next.iter().fold(0, |mut count, m| {count += m.count; count}) == 20);
        assert!(load_next.len() == 1);
        // println!("->> load_next:");
        // println!("count: {}", );
        // println!("msgs: {:#?}", load_next
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );
        let reload1 = room_chunks.reload();
        assert!(reload1.iter().fold(0, |mut count, m| {count += m.count; count}) == 20);
        assert!(reload1.len() == 1);
        // println!("->> reload1:");
        // println!("count: {}", reload1.iter().fold(0, |mut count, m| {count += m.count; count}));
        // println!("msgs: {:#?}", reload1
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );
        let load_next1 = room_chunks.load_next();
        assert!(load_next1.iter().fold(0, |mut count, m| {count += m.count; count}) == 40);
        assert!(load_next1.len() == 2);
        // println!("->> load_next1:");
        // println!("count: {}", load_next1.iter().fold(0, |mut count, m| {count += m.count; count}));
        // println!("msgs: {:#?}", load_next1
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );

        let reload2 = room_chunks.reload();
        assert!(reload2.iter().fold(0, |mut count, m| {count += m.count; count}) == 40);
        assert!(reload2.len() == 2);
        // println!("->> reload2:");
        // println!("count: {}", reload2.iter().fold(0, |mut count, m| {count += m.count; count}));
        // println!("msgs: {:#?}", reload2
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );

        let load_next2 = room_chunks.load_next();
        assert!(load_next2.iter().fold(0, |mut count, m| {count += m.count; count}) == 60);
        assert!(load_next2.len() == 3);
        // println!("->> load_next2:");
        // println!("count: {}", load_next2.iter().fold(0, |mut count, m| {count += m.count; count}));
        // println!("msgs: {:#?}", load_next2
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );

        let reload3 = room_chunks.reload();
        assert!(reload3.iter().fold(0, |mut count, m| {count += m.count; count}) == 60);
        assert!(reload3.len() == 3);
        // println!("->> reload3:");
        // println!("count: {}", reload3.iter().fold(0, |mut count, m| {count += m.count; count}));
        // println!("msgs: {:#?}", reload3
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );
        
        let load_next3 = room_chunks.load_next();
        assert!(load_next3.iter().fold(0, |mut count, m| {count += m.count; count}) == 80);
        assert!(load_next3.len() == 4);
        // println!("->> load_next3:");
        // println!("count: {}", load_next3.iter().fold(0, |mut count, m| {count += m.count; count}));
        // println!("msgs: {:#?}", load_next3
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );  
    }
}