use std::collections::BTreeMap;
use std::cell::Cell;

use floem::prelude::RwSignal;
use im::{vector, Vector};
use tracing_lite::{debug, info, trace, warn};
use ulid::Ulid;

use crate::{util::Id, view_data::msg::MsgViewData};

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

    /// State of the display BTreeMap.
    /// When requested to load msg/chunk and there is no msgs at all,
    /// display state stays false.
    pub display_state: Cell<bool>,
    /// Oldest loaded chunk (should be lower number).
    pub oldest_display_chunk_idx: Cell<u16>,
    /// Youngest loaded chunk (should be bigger number).
    pub youngest_display_chunk_idx: Cell<u16>,
}

// pub struct MsgDisplayState {
//     /// State of the display (tab) BTreeMap.
//     pub is_on_display: Cell<bool>,
//     /// Oldest loaded chunk (should be lower number).
//     pub oldest_display_chunk_idx: Cell<u16>,
//     /// Youngest loaded chunk (should be bigger number).
//     pub youngest_display_chunk_idx: Cell<u16>
// }


impl RoomMsgChunks {
    /// Create empty self from the room id.
    pub fn new(room: Id) -> Self {
        Self {
            room_id: room,
            ..Self::default()
        }
    }

    /// Creates Self from single message.
    pub fn new_from_single_msg(msg: MsgViewData) -> Self {
        let mut chunk = MsgChunk::default();
        let room_id = msg.room.clone();
        chunk.add_msg(msg);
        Self {
            room_id,
            total_msgs: 1,
            chunks_count: 1,
            chunks: vec!(chunk),
            oldest_display_chunk_idx: Cell::new(0),
            youngest_display_chunk_idx: Cell::new(0),
            display_state: Cell::new(false)
        }
    }

    /// Create new chunks from message map.
    pub fn new_from_msgs(msgs: BTreeMap<Ulid, MsgViewData>, room_id: Id) -> Self {
        let total_msgs = msgs.len() as u16;
        info!("total msgs: {total_msgs}");
        let chunks = {
            match total_msgs {
                1..=20 => {
                    trace!("1 to 20 msgs");
                    let mut chunk = vec!();
                    for each in msgs.into_values().rev() {
                        chunk.push(each);
                    }
                    vec![MsgChunk::new(chunk)]
                },
                21.. => {
                    trace!("more than 21 msgs");
                    let mut chunks = Vec::new();
                    let loops = msgs.len() / 20;
                    let mut iter_on_values = msgs.values().rev();
                    for n in 0..loops {
                        trace!("loop for {n} in {loops}");
                        let mut chunk = vec!();
                        for n in 0..20 {
                            trace!("chunks for {n} in 20");
                            if let Some(element) = iter_on_values.next() {
                                warn!("loop msg: {}", element.id.id);
                                chunk.push(element.clone());
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
            oldest_display_chunk_idx: Cell::new(0),
            chunks_count,
            chunks,
            youngest_display_chunk_idx: Cell::new(chunks_count),
            display_state: Cell::new(false)
        }
    }

    /// Add single [Msg] to the chunk (will not update display marks).
    pub fn append_new_msg(&mut self, msg: MsgViewData) {
        // Get the chunk with youngest msgs and check if full
        match self.chunks.last_mut() {
            Some(chunk) => {
                if chunk.count >= 20 {
                    trace!("fn: append_new_msg: chunk.count >= 20");
                    // -- Create new chunk
                    self.chunks.push(MsgChunk::new(vec!(msg)));
                    self.chunks_count += 1;
                    self.total_msgs += 1;
                    // let display_idx = self.oldest_display_chunk_idx.get();
                    // self.oldest_display_chunk_idx.set(display_idx.saturating_add(1));
                } else {
                    trace!("fn: append_new_msg: chunk.count IS NOT >= 20");
                    // -- Push onto existing chunk
                    chunk.add_msg(msg);
                    self.total_msgs += 1;
                }
            },
            None => {
                trace!("fn: append_new_msg: appending first msg!");
                self.chunks.push(MsgChunk::new(vec!(msg)));
                self.chunks_count += 1;
                self.total_msgs += 1;
                // self.oldest_display_chunk_idx.set(1)
            },
        }
    }

    /// Evaluate if any more not loaded chunks left in the room.
    /// TODO: check it!
    pub fn anymore_available(&self) -> bool {
        self.chunks_count.saturating_sub(1) > self.oldest_display_chunk_idx.get()
    }

    // // #[deprecated]
    // /// Load room chunks in range until `last_chunk_on_display` + one.
    // pub fn load_next(&self) -> &[MsgChunk] {
    //     debug!("fn: load_next");
    //     // -- Check how many chunks is loaded and return if no more left
    //     if self.chunks_count == 0 {
    //         trace!("fn: load_next: nothing to load");
    //         return &[]
    //     }
    //     // -- Load another one
    //     trace!("self.last_chunk_on_display: {}", self.oldest_display_chunk_idx.get());
    //     let range = {
    //         // -- Subtract 1 from chunk_count as index starts from 0, not 1
    //         let display_idx = self.oldest_display_chunk_idx.get();
    //         let chunks_count = self.chunks_count;
    //         debug!("fn: load_next: {} == {}", display_idx, chunks_count);
    //         // Case 1: already shown everything (dis_idx == count - 1)
    //         if display_idx == chunks_count {
    //             self.oldest_display_chunk_idx.set(display_idx.saturating_sub(1));
    //             return &self.chunks[(display_idx - 2) as usize..=(chunks_count - 1) as usize]
    //         }
    //         // Case 2: still unloaded chunks available (dis_idx < count - 1)
    //         match display_idx {
    //             0 => self.chunks.as_slice(),
    //             1 => {
    //                 self.oldest_display_chunk_idx.set(display_idx.saturating_sub(1));
    //                 self.chunks.as_slice()
    //             },
    //             other => {
    //                 self.oldest_display_chunk_idx.set(display_idx.saturating_sub(1));
    //                 &self.chunks[(other - 2) as usize..=(self.chunks_count - 1) as usize]
    //             }
    //         }
    //     };
    //     debug!("loaded data:\ntotal chunks: {}\nlast chunk msg count: {}",
    //         range.len(), range.last().unwrap().count
    //     );
    //     range
    // }

    // -------------- v2 impls --------------

    /// Only load yougest msg.
    pub fn load_youngest_msg(&self) -> Option<&MsgViewData> {
        match self.last_msg() {
            Some(msg) => {
                // Check idx of the msg chunk and if it matches display state
                let msg_chunk_idx = self.msg_chunk_idx(&msg.id.id);
                if self.youngest_display_chunk_idx.get() < msg_chunk_idx {
                    self.youngest_display_chunk_idx.set(msg_chunk_idx)
                }
                Some(msg)
            },
            None => None
        }
    }
    
    /// Get index of the [MsgChunk] containing given [Msg].
    pub fn msg_chunk_idx(&self, msg: &Ulid) -> u16 {
        self.chunks
            .iter()
            .position(|c| c.last.timestamp_ms() >= msg.timestamp_ms())
            .unwrap_or_default() as u16
    }
    
    /// Load everything from particular point onwards (without the `earliest`).
    pub fn load_new_content(&self, earliest: Option<Ulid>) -> Vec<MsgViewData> {
        match earliest {
            Some(last_loaded_msg) => {
                let mut msg_idx = None;
                // 1. Find chunk.
                let mut chunk_idx = self.chunks
                    .iter()
                    .position(|chunk| {
                        debug!("chunk: {} <= {}", chunk.last.timestamp_ms(), last_loaded_msg.timestamp_ms());
                        if chunk.last.timestamp_ms() >= last_loaded_msg.timestamp_ms() {
                            // 2. Find that msg.
                            msg_idx = chunk.msgs
                                .iter()
                                .position(|msg| {
                                    debug!("{} == {last_loaded_msg}", msg.id.id);
                                    msg.id.id == last_loaded_msg
                                });
                            true
                        } else {
                            false
                        }
                });
                // 3. Fetch everything from that point onwards.
                if let Some(cidx) = chunk_idx {
                    if let Some(midx) = msg_idx {
                        let mut chunk_idx = cidx;
                        let msg_idx = {
                            // -- If earliest msg was last, start from the next chunk
                            if midx < 19 { midx + 1 } else { chunk_idx += 1; 0 }
                        };
                        // info!("cidx: {cidx}, msdix: {midx}");
                        let mut fetched_msgs = Vec::with_capacity(20);
                        // -- Fetch rest of the chunk
                        let chunk = self.chunks.get(chunk_idx).unwrap(); // Safety: chunk_idx was check before
                        fetched_msgs = chunk.msgs[msg_idx..].to_vec();
                        // -- Adjust display trackers
                        self.set_display(chunk_idx as u16, self.chunks_count - 1);
                        // -- Return if that chunk is the last one
                        if self.chunks_count as usize == (chunk_idx + 1) {
                            return fetched_msgs
                        }
                        // -- If more chunks then fetch them one by one (excluding the one fetched)
                        for c in &self.chunks[chunk_idx + 1..] {
                            for m in &c.msgs[..] {
                                fetched_msgs.push(m.clone());
                            }
                        }
                        return fetched_msgs
                    }
                }
                Vec::with_capacity(0)
            },
            None => {
                let mut fetched_msgs = vec!();
                // -- Load only last chunk or 2 (if last is less than 15 msgs)
                match self.chunks_count {
                    0 => (),
                    1 => {
                        self.set_display(0, 0);
                        fetched_msgs.extend_from_slice(&self.chunks.last().unwrap().msgs);
                    },
                    other => {
                        if self.chunks.last().unwrap().count < 15 {
                            self.set_display(other - 2, other - 1);
                            fetched_msgs.extend_from_slice(&self.chunks.last().unwrap().msgs);
                        } else {
                            self.set_display(other - 1, other - 1);
                            let msgs = self.chunks[(other - 2) as usize..].iter().map(|c| c.msgs.clone()).flatten();
                            fetched_msgs.extend(msgs);
                        }
                    }
                }
                fetched_msgs
            }
        }
    }
    
    /// Load next 20 older msgs from the oldest loaded chunk. 
    pub fn load_older_chunk(&self) -> &[MsgViewData] {
        // -- Get display state
        match self.display_state.get() {
            false => {
                // -- All markers should be on 0 as that point
                match self.chunks_count {
                    0 => &[],
                    other => {
                        self.set_display(other - 1, other - 1);
                        &self.chunks.last().unwrap().msgs[..]
                    }
                }
            },
            true => {
                let previous_oldest_chunk_idx = self.oldest_display_chunk_idx.get();
                // -- Check if oldest loaded chunk is also last available
                if previous_oldest_chunk_idx == 0 && self.chunks_count != 0 {
                    return &[] // No more unloaded chunks
                }
                // // -- Case where previous time requested to load there was no msgs in chunks
                // if previous_oldest_chunk_idx == 0 && self.youngest_display_chunk_idx.get() == 0 && self.chunks_count != 0 {
                //     self.update_display_markers(self.chunks_count - 1, self.chunks_count - 1);
                //     return &self.chunks.last().unwrap().msgs[..]
                // }
                // -- Load one more chunk
                self.oldest_display_chunk_idx.set(previous_oldest_chunk_idx - 1);
                &self.chunks[(previous_oldest_chunk_idx - 1) as usize].msgs[..]
            }
        }
    }

    /// Sets display state to `false` and display idx's to 0.
    pub fn reset_display(&self) {
        self.display_state.set(false);
        self.youngest_display_chunk_idx.set(0);
        self.oldest_display_chunk_idx.set(0)
    }

    /// Sets display state to `true` and display idx's to provided values.
    pub fn set_display(&self, oldest_idx: u16, youngest_idx: u16) {
        self.display_state.set(true);
        self.youngest_display_chunk_idx.set(youngest_idx);
        self.oldest_display_chunk_idx.set(oldest_idx)
    }

    /// Sets display markers to provided values.
    pub fn update_display_markers(&self, oldest_idx: u16, youngest_idx: u16) {
        self.youngest_display_chunk_idx.set(youngest_idx);
        self.oldest_display_chunk_idx.set(oldest_idx)
    }
    
    // /// Reload loaded chunks (as a result of new/changed message).
    // pub fn reload(&self) -> &[MsgChunk] {
    //     debug!("fn: reload");
    //     // -- Check how many chunks is loaded and return if no more left
    //     if self.chunks_count == 0 {
    //         trace!("fn: reload: nothing to reload");
    //         return &[]
    //     }
    //     let display_idx = self.oldest_display_chunk_idx.get();
    //     trace!("self.last_chunk_on_display: {display_idx}");

    //     if self.chunks_count == 1 {
    //         trace!("fn: reload: just 1 chunk total");
    //         self.oldest_display_chunk_idx.set(0); // TODO: check if not already set
    //         return self.chunks.as_slice()
    //     }

    //     let reloaded = {
    //         // -- Subtract 1 from chunk_count as index starts from 0, not 1
    //         let chunks_count = self.chunks_count;
    //         let last_chunk_msg_count = self.chunks.last().unwrap().count;
    //         debug!("fn: reload: {} == {}", display_idx, chunks_count);
    //         // Case 1: already shown everything (dis_idx == count - 1)
    //         // if display_idx == chunks_count {
    //         //     return &self.chunks[(display_idx - 1) as usize..=(chunks_count - 1) as usize]
    //         // }
    //         // Case 2: still unloaded chunks available (dis_idx < count - 1)
    //         match display_idx {
    //             0 | 1 => self.chunks.as_slice(),
    //             other if last_chunk_msg_count > 18 => {
    //                 &self.chunks[(other - 1) as usize..=(self.chunks_count - 1) as usize]
    //             },
    //             other => {
    //                 self.oldest_display_chunk_idx.set(display_idx.saturating_sub(1));
    //                 &self.chunks[(other - 2) as usize..=(self.chunks_count - 1) as usize]
    //             },
    //         }
    //     };
    //     debug!("reloaded data:\ntotal chunks: {}\nlast chunk msg count: {}",
    //         reloaded.len(), reloaded.last().unwrap().count
    //     );
    //     reloaded
    // }

    /// Returns reference to last Msg inserted.
    pub fn last_msg(&self) -> Option<&MsgViewData> {
        debug!("fn: last_msg");
        if self.total_msgs == 0 { return None }
        if let Some(chunk) = self.chunks.last() {
            chunk.last_msg()
        } else {
            None
        }
    }

    /// Updates Self with given [MsgViewData].
    pub fn update_one(&mut self, msg: &MsgViewData) {
        debug!("fn: update_one");
        self.chunks
            .iter_mut()
            // .rev()
            .find_map(|chunk| {
                debug!("->> {:?} >= {:?}", chunk.last.timestamp_ms(), msg.id.id.timestamp_ms());
                if chunk.last.timestamp_ms() >= msg.id.id.timestamp_ms() {
                    trace!(" ..is `Some`");
                    for (idx, each_msg) in chunk.msgs.iter().enumerate() {
                        if each_msg.id.id == msg.id.id {
                            debug!("found msg({}) and index({idx})", each_msg.id.id.timestamp_ms());
                            return Some((chunk, idx))
                        }      
                        // error!("index_of returned `None`");
                    }
                }
                None
            })
            .map(|(chunk, idx)| {
                let old = chunk.msgs.get_mut(idx).unwrap();
                *old = msg.clone();
                debug!("updated chunk with msg");
                trace!("old: {}", old.msg.text.current);
                trace!("new: {}", chunk.msgs.get(15).unwrap().msg.text.current);
            });
    }

    /// Attempt to find [MsgViewData] from the provided id.
    pub fn find_msg(&self, id: Ulid) -> Option<&MsgViewData> {
        debug!("fn: find_msg");
        self.chunks
            .iter()
            .find_map(|chunk| {
                if chunk.last >= id {
                    for msg in &chunk.msgs {
                        if msg.id.id == id {
                            trace!("find_msg: Some({})", msg.id.id);
                            return Some(msg)
                        }
                    }
                };
                trace!("find_msg: None");
                None
            })
    }
}

// MARK: MsgChunk

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MsgChunk {
    /// Max msgs per chunk: 20 (for now).
    pub count: u8,
    /// All the stored msgs.
    pub msgs: Vec<MsgViewData>,
    /// Earliest msg stored in this MsgChunk.
    pub first: Ulid,
    /// Lastest msg stored in this MsgChunk.
    pub last: Ulid
}

impl MsgChunk {
    /// Construct new MsgChunk from the Vec<MsgViewData>.
    /// TODO: check if there is no more than 20 msgs in the vec.
    pub fn new(mut msgs: Vec<MsgViewData>) -> Self {
        let (first, last) = {
            if msgs.len() == 1 {
                let id = msgs.first().unwrap().id.id;
                (id, id)
            } else {
                msgs.sort();
                let f = msgs.first().unwrap().id.id;
                let l = msgs.last().unwrap().id.id;
                (f, l)
            }
        };
        Self {
            count: msgs.len() as u8,
            msgs,
            first,
            last
        }
    }

    pub fn new_v2(msgs: Vec<MsgViewData>) -> Option<Self> {
        match msgs.len() {
            0 | 21.. => { return None },
            1 => {
                let first_last = msgs[0].id.id;
                Some(Self {
                    count: 1,
                    msgs,
                    first: first_last,
                    last: first_last
                })
            },
            len @ 2..=20 => {
                let first = msgs.first()?.id.id;
                let last = msgs.first()?.id.id;
                Some(
                    Self {
                        count: len as u8,
                        msgs,
                        first,
                        last
                    }
                )
            },
        }
    }
    
    /// Add single [MsgViewData] onto back of the [Vector].
    pub fn add_msg(&mut self, msg: MsgViewData) {
        // -- Update last msg Ulid
        self.last = msg.id.id;
        // -- If msg is first also update first msg Ulid
        if self.count == 0 { self.first = msg.id.id }
        // -- Add msg
        self.msgs.push(msg);
        // -- Increase msg count
        self.count += 1;
    }

    /// Returns reference to last Msg inserted.
    pub fn last_msg(&self) -> Option<&MsgViewData> {
        self.msgs.last()
    }
}

// MARK: DisplayCh.


// /// Structure that holds range of chunks displayed in the room.
// #[derive(Clone, Debug, Default)]
// pub struct DisplayChunks {
//     pub total: u16,
//     /// Oldest loaded chunk (should be lower number).
//     pub start: u16,
//     /// Youngest loaded chunk (should be bigger number).
//     pub last: u16
// }

// impl DisplayChunks {
//     /// Set complete range of loaded chunks.
//     pub fn set_range(&mut self, start: u16, last: u16) {
//         self.start = start;
//         self.last = last;
//     }
//     /// Older chunk was loaded (`start` field).
//     pub fn loaded_older(&mut self) {
//         let new_val = self.start.saturating_sub(1);
//         self.start = new_val;
//     }
    
//     /// New chunk was added to the front (`last` field).
//     pub fn added_new_chunk(&mut self) {
//         let new_val = self.last.saturating_add(1);
//         self.last = new_val;
//     }

//     /// Deloded 1 or more old chunks.
//     pub fn deloaded_old_chunks(&mut self, no_of_deloaded: u16) {
//         self.start = no_of_deloaded;
//     }
// }

/// Chunk load cases.
#[derive(Clone, Debug)]
pub enum ChunkLoadCase {
    /// Nothing else to load: either all or nothing loaded.
    EverythingLoaded,
    /// Load first chunk.
    NothingLoaded,
    /// One chunk loaded, load next chunk.
    OneLoaded,
    /// Many chunks loaded, load next chunk.
    ManyLoaded,
}




#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::rc::Rc;
    use std::time::Duration;
    use tracing_lite::{trace, Subscriber};

    use crate::cont::acc::Account;
    use crate::util::{Id, Tb};
    use super::{MsgViewData, RoomMsgChunks};

    #[test]
    fn last_msg_test() {
        let act_room = Id::new(Tb::Room);
        let acc = Account {
            acc_id: Id::new(Tb::Acc),
            username: "Karol".into(),
            av: Rc::new(vec![]),
        };
        let msg = MsgViewData::new_from_click(act_room.clone(), &acc);
        let mut room_chunks = RoomMsgChunks::new_from_single_msg(msg.clone());
        assert_eq!(room_chunks.last_msg(), Some(&msg));
        // room_chunks.room_id = act_room.clone();
        let msg2 = MsgViewData::new_from_click(act_room.clone(), &acc);
        let msg3 = MsgViewData::new_from_click(act_room.clone(), &acc);
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
            let msg = MsgViewData::new_from_click(act_room.clone(), &acc);
            msgs_vec.push(msg);
        }
        // let len = msgs_vec.len();
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

    #[test]
    fn load_new_content_test() {
        Subscriber::new_with_max_level(tracing_lite::Level::INFO);
        let act_room = Id::new(Tb::Room);
        let acc = Account {
            acc_id: Id::new(Tb::Acc),
            username: "Karol".into(),
            av: Rc::new(vec![]),
        };
        let mut chunks = RoomMsgChunks::new(act_room.clone());

        let mut msgs_vec = VecDeque::with_capacity(80);
        for _ in 0..52 {
            std::thread::sleep(Duration::from_millis(2));
            let msg = MsgViewData::new_from_click(act_room.clone(), &acc);
            msgs_vec.push_back(msg);
        }

        let from_2_case = msgs_vec[0].id.id;
        let from_3_case = msgs_vec[1].id.id;
        let from_19_case = msgs_vec[18].id.id;
        let from_20_case = msgs_vec[19].id.id;
        let from_21_case = msgs_vec[20].id.id;
        let from_50_case = msgs_vec[49].id.id;
        let no_never_msg_case = msgs_vec[51].id.id;

        chunks.append_new_msg(msgs_vec.pop_front().unwrap());

        for msg in msgs_vec {
            chunks.append_new_msg(msg)
        }

        // ------------------------------------------------------------
        // -- from_2_case
        let from_2_case_res = chunks.load_new_content(Some(from_2_case));
        println!("from_2_case_res len: {}", from_2_case_res.len());
        println!("from_2_case_res display status: oldest idx: {}, yougest idx: {}",
            chunks.oldest_display_chunk_idx.get(),
            chunks.youngest_display_chunk_idx.get()
        );
        println!("from_2_case_res: {:#?}",
            // from_2_case_res.first().map(|m| m.msg.text.current.clone())
            from_2_case_res.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>()
        );
        assert_eq!(chunks.oldest_display_chunk_idx.get(), 0);
        assert_eq!(chunks.youngest_display_chunk_idx.get(), 2);
        assert_eq!(from_2_case_res.len(), 51);
        assert_eq!(from_2_case_res.first().unwrap().msg.text.current, String::from("Really important message no: 2"));

        // ------------------------------------------------------------
        // -- from_3_case
        let from_3_case_res = chunks.load_new_content(Some(from_3_case));
        println!("from_3_case_res len: {}", from_3_case_res.len());
        println!("from_3_case_res display status: oldest idx: {}, yougest idx: {}",
            chunks.oldest_display_chunk_idx.get(),
            chunks.youngest_display_chunk_idx.get()
        );
        println!("from_3_case_res: {:#?}",
            // from_3_case_res.first().map(|m| m.msg.text.current.clone())
            from_3_case_res.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>()
        );
        assert_eq!(chunks.oldest_display_chunk_idx.get(), 0);
        assert_eq!(chunks.youngest_display_chunk_idx.get(), 2);
        assert_eq!(from_3_case_res.len(), 50);
        assert_eq!(from_3_case_res.first().unwrap().msg.text.current, String::from("Really important message no: 3"));

        // ------------------------------------------------------------
        // -- from_19_case
        let from_19_case_res = chunks.load_new_content(Some(from_19_case));
        println!("from_19_case_res len: {}", from_19_case_res.len());
        println!("from_19_case_res display status: oldest idx: {}, yougest idx: {}",
            chunks.oldest_display_chunk_idx.get(),
            chunks.youngest_display_chunk_idx.get()
        );
        println!("from_19_case_res: {:#?}",
            // from_19_case_res.first().map(|m| m.msg.text.current.clone())
            from_19_case_res.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>()
        );
        assert_eq!(chunks.oldest_display_chunk_idx.get(), 0);
        assert_eq!(chunks.youngest_display_chunk_idx.get(), 2);
        assert_eq!(from_19_case_res.len(), 33);
        assert_eq!(from_19_case_res.first().unwrap().msg.text.current, String::from("Really important message no: 20"));

        // ------------------------------------------------------------
        // -- from_20_case
        let from_20_case_res = chunks.load_new_content(Some(from_20_case));
        println!("from_20_case_res len: {}", from_20_case_res.len());
        println!("from_20_case_res display status: oldest idx: {}, yougest idx: {}",
            chunks.oldest_display_chunk_idx.get(),
            chunks.youngest_display_chunk_idx.get()
        );
        println!("from_20_case_res: {:#?}",
            // from_20_case_res.first().map(|m| m.msg.text.current.clone())
            from_20_case_res.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>()
        );
        assert_eq!(chunks.oldest_display_chunk_idx.get(), 1);
        assert_eq!(chunks.youngest_display_chunk_idx.get(), 2);
        assert_eq!(from_20_case_res.len(), 32);
        assert_eq!(from_20_case_res.first().unwrap().msg.text.current, String::from("Really important message no: 21"));

        // ------------------------------------------------------------
        // -- from_21_case
        let from_21_case_res = chunks.load_new_content(Some(from_21_case));
        println!("from_21_case_res len: {}", from_21_case_res.len());
        println!("from_21_case_res display status: oldest idx: {}, yougest idx: {}",
            chunks.oldest_display_chunk_idx.get(),
            chunks.youngest_display_chunk_idx.get()
        );
        println!("from_21_case_res: {:#?}",
            // from_21_case_res.first().map(|m| m.msg.text.current.clone())
            from_21_case_res.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>()
        );
        assert_eq!(chunks.oldest_display_chunk_idx.get(), 1);
        assert_eq!(chunks.youngest_display_chunk_idx.get(), 2);
        assert_eq!(from_21_case_res.len(), 31);
        assert_eq!(from_21_case_res.first().unwrap().msg.text.current, String::from("Really important message no: 22"));

        // ------------------------------------------------------------
        // -- from_50_case
        let from_50_case_res = chunks.load_new_content(Some(from_50_case));
        println!("from_50_case_res len: {}", from_50_case_res.len());
        println!("from_50_case_res display status: oldest idx: {}, yougest idx: {}",
            chunks.oldest_display_chunk_idx.get(),
            chunks.youngest_display_chunk_idx.get()
        );
        println!("from_50_case_res: {:#?}",
            // from_50_case_res.first().map(|m| m.msg.text.current.clone())
            from_50_case_res.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>()
        );
        assert_eq!(chunks.oldest_display_chunk_idx.get(), 2);
        assert_eq!(chunks.youngest_display_chunk_idx.get(), 2);
        assert_eq!(from_50_case_res.len(), 2);
        assert_eq!(from_50_case_res.first().unwrap().msg.text.current, String::from("Really important message no: 51"));

        // ------------------------------------------------------------
        // -- only_last_case
        let no_never_msg_case_res = chunks.load_new_content(Some(no_never_msg_case));
        println!("no_never_msg_case_res len: {}", no_never_msg_case_res.len());
        println!("no_never_msg_case_res display status: oldest idx: {}, yougest idx: {}",
            chunks.oldest_display_chunk_idx.get(),
            chunks.youngest_display_chunk_idx.get()
        );
        println!("no_never_msg_case_res: {:#?}",
            // no_never_msg_case_res.first().map(|m| m.msg.text.current.clone())
            no_never_msg_case_res.iter().map(|m| m.msg.text.current.clone()).collect::<Vec<_>>()
        );
        assert_eq!(chunks.oldest_display_chunk_idx.get(), 2);
        assert_eq!(chunks.youngest_display_chunk_idx.get(), 2);
        assert_eq!(no_never_msg_case_res.len(), 0);
        assert_eq!(no_never_msg_case_res.first(), None);
    }

    #[test]
    fn msg_chunk_idx_test() {
        let act_room = Id::new(Tb::Room);
        let acc = Account {
            acc_id: Id::new(Tb::Acc),
            username: "Karol".into(),
            av: Rc::new(vec![]),
        };
        let mut chunks = RoomMsgChunks::new(act_room.clone());

        let mut msgs_vec = VecDeque::with_capacity(80);
        for _ in 0..52 {
            std::thread::sleep(Duration::from_millis(2));
            let msg = MsgViewData::new_from_click(act_room.clone(), &acc);
            msgs_vec.push_back(msg);
        }

        let case1 = msgs_vec[1].id.id;
        let case19 = msgs_vec[19].id.id;
        let case20 = msgs_vec[20].id.id;
        let case49 = msgs_vec[49].id.id;
        let case51 = msgs_vec[51].id.id;

        for msg in msgs_vec {
            chunks.append_new_msg(msg)
        }

        assert_eq!(chunks.msg_chunk_idx(&case1), 0);
        assert_eq!(chunks.msg_chunk_idx(&case19), 0);
        assert_eq!(chunks.msg_chunk_idx(&case20), 1);
        assert_eq!(chunks.msg_chunk_idx(&case49), 2);
        assert_eq!(chunks.msg_chunk_idx(&case51), 2);
    }

    #[test]
    fn load_older_chunk_test() {
        Subscriber::new_with_max_level(tracing_lite::Level::DEBUG);
        let act_room = Id::new(Tb::Room);
        let acc = Account {
            acc_id: Id::new(Tb::Acc),
            username: "Karol".into(),
            av: Rc::new(vec![]),
        };
        let mut chunks = RoomMsgChunks::new(act_room.clone());
        assert!(chunks.load_older_chunk().is_empty());
        assert_eq!(chunks.display_state.get(), false);
        assert_eq!(chunks.oldest_display_chunk_idx.get(), 0);
        assert_eq!(chunks.youngest_display_chunk_idx.get(), 0);

        let mut msgs_vec = VecDeque::with_capacity(80);
        for _ in 0..52 {
            std::thread::sleep(Duration::from_millis(2));
            let msg = MsgViewData::new_from_click(act_room.clone(), &acc);
            msgs_vec.push_back(msg);
        }
        for msg in msgs_vec {
            chunks.append_new_msg(msg)
        }
        let mut display_vec = VecDeque::with_capacity(52);

        for msg in chunks.load_older_chunk().iter().rev() {
            display_vec.push_front(msg);
        }
        println!("oldest_idx: {}, youngest_idx: {}", chunks.oldest_display_chunk_idx.get(), chunks.youngest_display_chunk_idx.get());
        assert_eq!(display_vec.len(), 12);
        
        for msg in chunks.load_older_chunk().iter().rev() {
            display_vec.push_front(msg);
        }
        println!("oldest_idx: {}, youngest_idx: {}", chunks.oldest_display_chunk_idx.get(), chunks.youngest_display_chunk_idx.get());
        assert_eq!(display_vec.len(), 32);
        
        for msg in chunks.load_older_chunk().iter().rev() {
            display_vec.push_front(msg);
        }
        println!("oldest_idx: {}, youngest_idx: {}", chunks.oldest_display_chunk_idx.get(), chunks.youngest_display_chunk_idx.get());
        assert_eq!(display_vec.len(), 52);
        
        for msg in chunks.load_older_chunk().iter().rev() {
            display_vec.push_front(msg);
        }
        println!("oldest_idx: {}, youngest_idx: {}", chunks.oldest_display_chunk_idx.get(), chunks.youngest_display_chunk_idx.get());
        assert_eq!(display_vec.len(), 52);

    }
}