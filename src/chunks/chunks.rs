use std::{cell::Cell, collections::BTreeMap};

use tracing_lite::{debug, info, trace, warn};
use ulid::Ulid;

use crate::{util::Id, view_data::msg::MsgViewData};

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
    /// When `with_limit` is `yes`, only youngest chunk or 20 msgs with be fetched.
    pub fn load_new_content(&self, earliest: Option<Ulid>, with_limit: bool) -> Vec<MsgViewData> {
        // trace!("fn: load_new_content: earliest is: {earliest:?}");
        match earliest {
            Some(last_loaded_msg) => {
                let mut msg_idx = None;
                // 1. Find chunk.
                let chunk_idx = self.chunks
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

                        // -- Return if limit is applied
                        if with_limit {
                            // -- Assess if length of the new content is more that one chunk
                            if self.chunks_count - chunk_idx as u16 > 1 {
                                // -- Check if last chunk is more that 15 msgs
                                let last = self.chunks.last().unwrap();
                                if last.count > 15 {
                                    // -- Just fetch last chunk (as is sufficientely big)
                                    fetched_msgs = last.msgs[msg_idx..].to_vec();
                                    self.set_display(chunk_idx as u16, self.chunks_count - 1);
                                    return fetched_msgs
                                }
                                // -- Fetch last 2 chunks (1 whole or from the new content idx)
                                let fetch_idx = self.chunks_count as usize - 2;
                                let fetch_full = chunk_idx != fetch_idx;
                                self.set_display(fetch_idx as u16, self.chunks_count - 1);

                                for c in &self.chunks[fetch_idx..] {
                                    // TODO: impl check for earlier chunk if all msgs can be fetched
                                    //       or just those after msg_idx
                                    for m in &c.msgs[..] { 
                                        fetched_msgs.push(m.clone());
                                    }
                                }    
                            }
                        }
                        
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
                let mut fetched_msgs = Vec::with_capacity(20);
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

    /// Evaluate if need to apply focus on display msg vector.
    pub fn need_focus(&self) -> Option<usize> {
        // Return if less that 2 chunks
        if self.chunks_count < 2 { return None }
        // Return if last chunk have less than 18 messages
        if self.chunks.last().unwrap().count < 18 { return None }
        // Calculate index


        todo!()
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