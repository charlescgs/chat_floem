use std::collections::BTreeMap;
use std::cell::Cell;
use std::rc::Rc;

use floem::prelude::*;
use im::{vector, Vector};
use tracing_lite::{debug, error, info, trace, warn};
use ulid::Ulid;

use crate::util::Id;
use super::msg::MsgCtx;

// MARK: RoomChunks

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
    pub chunks: Vec<MsgChunk>,
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
            chunks: vec!(chunk),
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
                    vec![MsgChunk::new(chunk)]
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
                        chunks.push(MsgChunk::new(chunk));
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
                    trace!("fn: append_new_msg: chunk.count >= 20");
                    // -- Create new chunk
                    self.chunks.push(MsgChunk::new(Vector::unit(msg)));
                    self.chunks_count += 1;
                    self.total_msgs += 1;
                    
                    let mut display_idx = self.last_chunk_on_display.get();
                    self.last_chunk_on_display.set(display_idx.saturating_add(1));
                } else {
                    trace!("fn: append_new_msg: chunk.count IS NOT >= 20");
                    // -- Push onto existing chunk
                    chunk.add_msg(msg);
                    self.total_msgs += 1;
                }
            },
            None => {
                trace!("fn: append_new_msg: chunks.last_mut() is None");
                self.chunks.push(MsgChunk::new(Vector::unit(msg)));
                self.chunks_count += 1;
                self.total_msgs += 1;
                self.last_chunk_on_display.set(1)
            },
        }
    }

    /// Evaluate if any more not loaded chunks left in the room.
    /// TODO: check it!
    pub fn anymore_available(&self) -> bool {
        self.chunks_count > self.last_chunk_on_display.get()
    }

    /// Load room chunks in range until `last_chunk_on_display` + one.
    pub fn load_next(&self) -> &[MsgChunk] {
        debug!("fn: load_next");
        // -- Check how many chunks is loaded and return if no more left
        if self.chunks_count == 0 {
            trace!("fn: load_next: nothing to load");
            return &[]
        }
        // -- Load another one
        trace!("self.last_chunk_on_display: {}", self.last_chunk_on_display.get());
        let range = {
            // -- Subtract 1 from chunk_count as index starts from 0, not 1
            let display_idx = self.last_chunk_on_display.get();
            let chunks_count = self.chunks_count;
            debug!("fn: load_next: {} == {}", display_idx, chunks_count);
            // Case 1: already shown everything (dis_idx == count - 1)
            if display_idx == chunks_count {
                self.last_chunk_on_display.set(display_idx.saturating_sub(1));
                return &self.chunks[(display_idx - 2) as usize..=(chunks_count - 1) as usize]
            }
            // Case 2: still unloaded chunks available (dis_idx < count - 1)
            match display_idx {
                0 => self.chunks.as_slice(),
                1 => {
                    self.last_chunk_on_display.set(display_idx.saturating_sub(1));
                    self.chunks.as_slice()
                },
                other => {
                    self.last_chunk_on_display.set(display_idx.saturating_sub(1));
                    &self.chunks[(other - 2) as usize..=(self.chunks_count - 1) as usize]
                }
            }
        };
        debug!("loaded data:\ntotal chunks: {}\nlast chunk msg count: {}",
            range.len(), range.last().unwrap().count
        );
        range
    }
    
    /// Reload loaded chunks (as a result of new/changed message).
    pub fn reload(&self) -> &[MsgChunk] {
        debug!("fn: reload");
        // -- Check how many chunks is loaded and return if no more left
        if self.chunks_count == 0 {
            trace!("fn: reload: nothing to reload");
            return &[]
        }
        // -- Change slice range to 1 if display was 0
        let mut display_idx = self.last_chunk_on_display.get();
        trace!("self.last_chunk_on_display: {display_idx}");

        if self.chunks_count == 1 {
            trace!("fn: reload: just 1 chunk total");
            self.last_chunk_on_display.set(0); // TODO: check if not already set
            return self.chunks.as_slice()
        }

        let reloaded = {
            // -- Subtract 1 from chunk_count as index starts from 0, not 1
            let chunks_count = self.chunks_count;
            debug!("fn: reload: {} == {}", display_idx, chunks_count);
            // Case 1: already shown everything (dis_idx == count - 1)
            // if display_idx == chunks_count {
            //     return &self.chunks[(display_idx - 1) as usize..=(chunks_count - 1) as usize]
            // }
            // Case 2: still unloaded chunks available (dis_idx < count - 1)
            match display_idx {
                0 | 1 => self.chunks.as_slice(),
                other => &self.chunks[(other - 1) as usize..=(self.chunks_count - 1) as usize]
            }
        };
        debug!("reloaded data:\ntotal chunks: {}\nlast chunk msg count: {}",
            reloaded.len(), reloaded.last().unwrap().count
        );
        reloaded
    }

    /// Returns reference to last Msg inserted.
    pub fn last_msg(&self) -> Option<&MsgCtx> {
        debug!("fn: last_msg");
        if self.total_msgs == 0 { return None }
        if let Some(chunk) = self.chunks.last() {
            chunk.last_msg()
        } else {
            None
        }
    }

    /// Updates Self with given [MsgCtx].
    pub fn update_one(&mut self, msg: &MsgCtx) {
        debug!("fn: update_one");
        self.chunks
            .iter_mut()
            // .rev()
            .find_map(|chunk| {
                debug!("->> {:?} >= {:?}", chunk.last.unwrap().timestamp_ms(), msg.id.id.timestamp_ms());
                if let Some(last) = chunk.last {
                    if last.timestamp_ms() >= msg.id.id.timestamp_ms() {
                        trace!(" ..is `Some`");
                        for (idx, each_msg) in chunk.msgs.iter().enumerate() {
                            if each_msg.id.id == msg.id.id {
                                debug!("found msg({}) and index({idx})", each_msg.id.id.timestamp_ms());
                                return Some((chunk, idx))
                            }      
                            // error!("index_of returned `None`");
                        }
                    }
                }
                trace!(" ..is `None`");
                None
            })
            .map(|(chunk, idx)| {
                let old = chunk.msgs.set(idx, msg.clone());
                debug!("updated chunk with msg");
                trace!("old: {}", old.msg.text.current);
                trace!("new: {}", chunk.msgs.get(15).unwrap().msg.text.current);
            });
    }

    pub fn find_msg(&self, id: Ulid) -> Option<&MsgCtx> {
        debug!("fn: find_msg");
        self.chunks
            .iter()
            .find_map(|chunk| {
                if chunk.last >= Some(id) {
                    for msg in &chunk.msgs {
                        if msg.id.id == id {
                            trace!("find_msg: Some({})", msg.id.id);
                            return Some(msg)
                        }
                    }
                };
                trace!("find_msg: None");
                // return 'f: false;
                None
            })
    }
}

// MARK: MsgChunk

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MsgChunk {
    /// Max msgs per chunk: 20 (for now).
    pub count: u8,
    pub msgs: Vector<MsgCtx>,

    /// Earliest msg stored in this MsgChunk.
    pub first: Option<Ulid>,
    /// Lastest msg stored in this MsgChunk.
    pub last: Option<Ulid>,
}

impl MsgChunk {
    // Construct new MsgChunk from the Vector<MsgCtx>.
    pub fn new(mut msgs: Vector<MsgCtx>) -> Self {
        let (first, last) = {
            if msgs.len() == 1 {
                let id = msgs.front().unwrap().id.id;
                (id, id)
            } else {
                // for each in &msgs {
                //     println!("{}", each.msg.text.current)
                // }
                msgs.sort();
                // for each in &msgs {
                //     println!("{}", each.msg.text.current)
                // }
                let f = msgs.front().unwrap().id.id;
                let l = msgs.last().unwrap().id.id;
                (f, l)
            }
        };

        Self {
            count: msgs.len() as u8,
            msgs,
            first: Some(first),
            last: Some(last)
        }
    }

    /// Add single [MsgCtx] onto back of the [Vector].
    pub fn add_msg(&mut self, msg: MsgCtx) {
        // -- Update last msg Ulid
        self.last = Some(msg.id.id);
        // -- If msg is first also update first msg Ulid
        if self.count == 0 { self.first = Some(msg.id.id) }
        // -- Add msg
        self.msgs.push_back(msg);
        // -- Increase msg count
        self.count += 1;
    }

    /// Returns reference to last Msg inserted.
    pub fn last_msg(&self) -> Option<&MsgCtx> {
        self.msgs.last()
    }
}


#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::time::Duration;
    use tracing_lite::{debug, trace, Subscriber};

    use crate::cont::acc::Account;
    use crate::util::{Id, Tb};
    use super::{MsgCtx, RoomMsgChunks};

    #[test]
    fn chunks_load_reload_test() {
        Subscriber::new_with_max_level(tracing_lite::Level::DEBUG);
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
        let reload_count = reload.iter().fold(0, |mut count, m| {count += m.count; count});
        debug!("reload: {reload_count} == 20");
        assert!(reload_count == 20);
        assert!(reload.len() == 1);
        // println!("->> reload:");
        // println!("count: {}", );
        // println!("msgs: {:#?}", reload
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );
        let load_next = room_chunks.load_next();
        let load_next_count = load_next.iter().fold(0, |mut count, m| {count += m.count; count});
        debug!("load_next: {load_next_count} == 40");
        assert!(load_next_count == 40);
        assert!(load_next.len() == 2);
        // println!("->> load_next:");
        // println!("count: {}", );
        // println!("msgs: {:#?}", load_next
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );
        let reload1 = room_chunks.reload();
        let reload1_count = reload1.iter().fold(0, |mut count, m| {count += m.count; count});
        debug!("reload1: {reload1_count} == 40");
        assert!(reload1_count == 40);
        assert!(reload1.len() == 2);
        // println!("->> reload1:");
        // println!("count: {}", reload1.iter().fold(0, |mut count, m| {count += m.count; count}));
        // println!("msgs: {:#?}", reload1
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );
        let load_next1 = room_chunks.load_next();
        let load_next1_count = load_next1.iter().fold(0, |mut count, m| {count += m.count; count});
        debug!("load_next1: {load_next1_count} == 60");
        assert!(load_next1_count == 60);
        assert!(load_next1.len() == 3);
        // println!("->> load_next1:");
        // println!("count: {}", load_next1.iter().fold(0, |mut count, m| {count += m.count; count}));
        // println!("msgs: {:#?}", load_next1
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );

        let reload2 = room_chunks.reload();
        let reload2_count = reload2.iter().fold(0, |mut count, m| {count += m.count; count});
        debug!("reload2: {reload2_count} == 60");
        assert!(reload2_count == 60);
        assert!(reload2.len() == 3);
        // println!("->> reload2:");
        // println!("count: {}", reload2.iter().fold(0, |mut count, m| {count += m.count; count}));
        // println!("msgs: {:#?}", reload2
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );

        let load_next2 = room_chunks.load_next();
        let load_next2_count = load_next2.iter().fold(0, |mut count, m| {count += m.count; count});
        debug!("load_next2: {load_next2_count} == 80");
        assert!(load_next2_count == 80);
        assert!(load_next2.len() == 4);
        // println!("->> load_next2:");
        // println!("count: {}", load_next2.iter().fold(0, |mut count, m| {count += m.count; count}));
        // println!("msgs: {:#?}", load_next2
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );

        let reload3 = room_chunks.reload();
        let reload3_count = reload3.iter().fold(0, |mut count, m| {count += m.count; count});
        debug!("reload3: {reload3_count} == 80");
        assert!(reload3_count == 80);
        assert!(reload3.len() == 4);
        // println!("->> reload3:");
        // println!("count: {}", reload3.iter().fold(0, |mut count, m| {count += m.count; count}));
        // println!("msgs: {:#?}", reload3
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );
        
        let load_next3 = room_chunks.load_next();
        let load_next3_count = load_next3.iter().fold(0, |mut count, m| {count += m.count; count});
        debug!("load_next3: {load_next3_count} == 80");
        assert!(load_next3_count == 80);
        assert!(load_next3.len() == 4);
        // println!("->> load_next3:");
        // println!("count: {}", load_next3.iter().fold(0, |mut count, m| {count += m.count; count}));
        // println!("msgs: {:#?}", load_next3
        //     .iter()
        //     .map(|mc| mc.msgs.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>())
        //     .collect::<Vec<_>>()
        // );  
    }
    #[test]
    fn last_msg_test() {
        let act_room = Id::new(Tb::Room);
        let acc = Account {
            acc_id: Id::new(Tb::Acc),
            username: "Karol".into(),
            av: Rc::new(vec![]),
        };
        let msg = MsgCtx::new_from_click(&act_room, &acc);
        let mut room_chunks = RoomMsgChunks::new_from_single_msg(msg.clone());
        assert_eq!(room_chunks.last_msg(), Some(&msg));
        // room_chunks.room_id = act_room.clone();
        let msg2 = MsgCtx::new_from_click(&act_room, &acc);
        let msg3 = MsgCtx::new_from_click(&act_room, &acc);
        room_chunks.append_new_msg(msg2);
        room_chunks.append_new_msg(msg3.clone());
        assert_eq!(room_chunks.last_msg(), Some(&msg3));
    }
    #[test]
    fn update_one_test() {
        // let id1 = Id::new(Tb::Msg);
        // std::thread::sleep(Duration::from_millis(2));
        // let id2 = Id::new(Tb::Msg);
        // std::thread::sleep(Duration::from_millis(2));
        // let id3 = Id::new(Tb::Msg);
        // std::thread::sleep(Duration::from_millis(2));
        // assert!(id1 < id2);
        // assert!(id1 <= id2);
        // assert!(id1 == id1);
        // assert!(id2 < id3);
        // assert!(id1 < id3);
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
        let msg_idx = 15;
        let mut msg_to_upt = msgs_vec.get(msg_idx).unwrap().clone();
        let msg_id = msg_to_upt.id.id;
        let mut msg_upt = msg_to_upt.msg.clone();
        Rc::make_mut(&mut msg_upt).text.current = String::from("Edited msg");
        msg_to_upt.msg = msg_upt;
        trace!("1: updated_text: {}", msg_to_upt.msg.text.current);
        // 2. Insert them into RoomMsgChunks
        let mut room_chunks = RoomMsgChunks::default();
        room_chunks.room_id = act_room.clone();
        for each in &msgs_vec {
            room_chunks.append_new_msg(each.clone());
        }
        // 3. Print it and assert.
        // println!("->> Before:");
        // println!("room_id: {}", room_chunks.room_id);
        // println!("anymore_available: {}", room_chunks.anymore_available());
        // // println!("msgs_vec len: {len}");
        // println!("chunks_count: {}", room_chunks.chunks_count);
        // println!("last_chunk_on_display: {}", room_chunks.last_chunk_on_display.get());
        // println!("total_msgs: {}", room_chunks.total_msgs);

        room_chunks.update_one(&msg_to_upt);
        let fetched = room_chunks.find_msg(msg_id).unwrap();
        // println!("updated: {}", msg_to_upt.msg.text.current);
        // println!("fetched_after_update: {}", fetched.msg.text.current);
        assert_eq!(&fetched.msg.text.current, &msg_to_upt.msg.text.current)
    }
}