use std::borrow::BorrowMut;
use std::collections::VecDeque;

use im::{vector, Vector};
use tracing_lite::trace;
use ulid::Ulid;

use crate::util::{Id, Tb};
use crate::view_data::msg::MsgViewData;
use super::RoomMsgChunks;




#[derive(Clone, Debug, Default, PartialEq)]
pub enum DisStatus {
    #[default]
    AllVisible,
    PartiallyHidden(u16, Ulid)
}


/// Structure that holds range of chunks displayed in the room.
#[derive(Clone, Debug, Default)]
pub struct DisplayChunks {
    /// All appended msgs.
    pub total_stored: u16,
    /// Storage vector.
    pub vec: Vector<MsgViewData>,
    /// Oldest loaded chunk (should be lower number).
    pub start: (u16, Ulid),
    /// Youngest loaded chunk (should be bigger number).
    pub last: (u16, Ulid),
    /// If any msgs are hidden.
    /// Anything before that index/msg will not be loaded.
    pub status: DisStatus
}

impl DisplayChunks {
    /// New empty instance of Self.
    pub fn new() -> Self {
        Self {
            total_stored: 0,
            vec: vector!(),
            start: (0, Ulid::nil()),
            last: (0, Ulid::nil()),
            status: DisStatus::default()
        }
    }

    /// Fetch lastest / youngest msg from the chunks.
    pub fn append_new(&mut self, new: MsgViewData) {
        // Empty case
        if self.total_stored == 0 {
            self.start = (0, new.id.id);
            self.last = (0, new.id.id);
        }
        // Only 1 case
        if self.total_stored == 1 {
            self.last = (1, new.id.id);
        }
        // Many case
        self.total_stored += 1;
        self.last = (self.total_stored - 1, new.id.id);
        self.vec.push_back(new);
    }

    /// Fetch multiple msgs younger than last-on-display from the [Chunks](super).
    pub fn append_many(&mut self, many: &[MsgViewData]) {
        if many.is_empty() { return }
        let vector = Vector::from(many);
        let appended_count = vector.len() as u16;
        // Empty case
        if self.total_stored == 0 {
            self.vec = vector;
            // SAFETY: vec was checked if empty before
            self.start = (0, self.vec.front().unwrap().id.id);
            self.last = (0, self.vec.back().unwrap().id.id);
            self.total_stored = appended_count;
        } else {
            // Many case
            self.vec.append(vector);
            self.last = (self.vec.len() as u16, self.vec.back().unwrap().id.id);
            self.total_stored += appended_count;
        }
    }

    /// Fetch edited msg and replace it.
    pub fn msg_edited(&mut self, edited: MsgViewData) {
        // Check if given msg is first or last
        let edited_id = edited.ulid();
        if edited_id == self.start.1 {
            *self.vec.front_mut().unwrap() = edited;
        } else if edited_id == self.last.1 {
            *self.vec.back_mut().unwrap() = edited;
        } else {
            self.vec
                .iter_mut()
                .find(|msg| msg.ulid() == edited_id)
                .and_then(|msg| Some(*msg = edited));
        }
    }
    
    /// Remove deleted msg.
    pub fn msg_removed(&mut self, del: Ulid) {
        if let Some(msg) = self.vec.iter().find(|msg| msg.ulid() == del) {
            if let Some(idx) = self.vec.index_of(msg) {
                let r = match idx {
                    // First if only one msg stored
                    0 if self.total_stored == 1 => {
                        self.start.1 = Ulid::nil();
                        self.last.1 = Ulid::nil();
                        self.vec.pop_front().unwrap()
                    },
                    // First if 2 or more
                    0 => {
                        self.start.1 = self.vec.get(1).unwrap().ulid();
                        self.vec.pop_front().unwrap()
                    },
                    // Last
                    i if i as u16 == self.total_stored - 1 => {
                        let new_last = self.vec.get(i - 1).unwrap().ulid();
                        self.last = (i as u16 - 1, new_last);
                        self.vec.pop_back().unwrap()
                    },
                    // Other case
                    _ => {
                        self.vec.remove(idx)
                    }
                };
                assert!(r.ulid() == del);
                self.total_stored -= 1;
            }
        }
    }

    /// Hide msgs older than 20 or do nothing.
    pub fn hide_older(&mut self) {
        match self.total_stored {
            0..20 => self.status = DisStatus::AllVisible,
            20.. => {
                let hide_point = self.vec.get(19).unwrap().ulid();
                self.status = DisStatus::PartiallyHidden(19, hide_point)
            }
        }
    }

    /// Fetch another full [Chunk](super).
    pub fn append_older_chunk(&mut self, chunk: &[MsgViewData]) {
        let chunk_len = chunk.len();
        chunk.iter().rev().map(|msg| self.vec.push_front(msg.clone()));
        self.start.1 = self.vec.front().unwrap().ulid();
        self.total_stored += chunk_len as u16;
    }
    
    /// Fetch another full [Chunk](super).
    pub fn append_older_chunk_alt(&mut self, chunk: &[MsgViewData]) {
        let chunk_len = chunk.len();
        // Create new Vector from the slice
        let mut new_front = Vector::from(chunk);
        // Clone currect self
        let vec = self.vec.clone();
        // Append current self on the end of new vector
        new_front.append(vec);
        // Override self with joined collection
        self.vec = new_front;
        self.start.1 = self.vec.front().unwrap().ulid();
        self.total_stored += chunk_len as u16;
    }
}

impl IntoIterator for DisplayChunks {
    type Item = MsgViewData;

    type IntoIter = im::vector::ConsumingIter<MsgViewData>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

// /// Chunk load cases.
// #[derive(Clone, Debug)]
// pub enum ChunkLoadCase {
//     /// Nothing else to load: either all or nothing loaded.
//     EverythingLoaded,
//     /// Load first chunk.
//     NothingLoaded,
//     /// One chunk loaded, load next chunk.
//     OneLoaded,
//     /// Many chunks loaded, load next chunk.
//     ManyLoaded,
// }

#[test]
fn display_append_edit_test() {
    let act_room = Id::new(Tb::Room);
    let acc = crate::cont::acc::Account {
        acc_id: Id::new(Tb::Acc),
        username: "Karol".into(),
        av: std::rc::Rc::new(vec![]),
    };
    let mut chunks = RoomMsgChunks::new(act_room.clone());
    let mut msg_vec = VecDeque::new();
    for _ in 0..22 {
        std::thread::sleep(std::time::Duration::from_millis(2));
        let msg = MsgViewData::new_from_click(act_room.clone(), &acc);
        msg_vec.push_back(msg);
    }
    let mut display = DisplayChunks::new();
    // ------------------
    let mut edited = msg_vec.get(5).cloned().unwrap();

    // APPEND NEW
    display.append_new(msg_vec.get(0).cloned().unwrap());
    assert_eq!(display.total_stored, 1);

    // APPEND MANY
    display.append_many(&msg_vec.make_contiguous()[1..]);
    assert_eq!(display.total_stored, 22);
    
    // MSG EDITED
    display.msg_edited(edited.clone());
    assert_eq!(display.total_stored, 22);
    assert_eq!(display.vec.get(5).unwrap().id, edited.id);
}

#[test]
fn display_msg_removed_test() {
    let act_room = Id::new(Tb::Room);
    let acc = crate::cont::acc::Account {
        acc_id: Id::new(Tb::Acc),
        username: "Karol".into(),
        av: std::rc::Rc::new(vec![]),
    };
    let mut chunks = RoomMsgChunks::new(act_room.clone());
    let mut msg_vec = VecDeque::new();
    for _ in 0..22 {
        std::thread::sleep(std::time::Duration::from_millis(2));
        let msg = MsgViewData::new_from_click(act_room.clone(), &acc);
        msg_vec.push_back(msg);
    }
    let mut display = DisplayChunks::new();
    // ------------------
    let del_first = msg_vec.get(0).cloned().unwrap();
    let del_second = msg_vec.get(1).cloned().unwrap();
    let del_third = msg_vec.get(2).cloned().unwrap();

    // Remove when empty
    assert_eq!(display.total_stored, 0);
    display.msg_removed(del_first.ulid());
    assert_eq!(display.total_stored, 0);
    // Remove when 1
    display.append_new(del_first.clone());
    assert_eq!(display.total_stored, 1);
    display.msg_removed(del_first.ulid());
    assert_eq!(display.total_stored, 0);
    // Remove when 2
    display.append_many(&msg_vec.make_contiguous()[0..2]);
    assert_eq!(display.total_stored, 2);
    display.msg_removed(del_first.ulid());
    assert_eq!(display.total_stored, 1);
    display.msg_removed(del_second.ulid());
    assert_eq!(display.total_stored, 0);
    // Remove when 3
    display.append_many(&msg_vec.make_contiguous()[0..3]);
    assert_eq!(display.total_stored, 3);
    display.msg_removed(del_third.ulid());
    assert_eq!(display.total_stored, 2);
}

#[test]
fn display_hide_older_test() {
    let act_room = Id::new(Tb::Room);
    let acc = crate::cont::acc::Account {
        acc_id: Id::new(Tb::Acc),
        username: "Karol".into(),
        av: std::rc::Rc::new(vec![]),
    };
    let mut chunks = RoomMsgChunks::new(act_room.clone());
    let mut msg_vec = VecDeque::new();
    for _ in 0..42 {
        std::thread::sleep(std::time::Duration::from_millis(2));
        let msg = MsgViewData::new_from_click(act_room.clone(), &acc);
        msg_vec.push_back(msg);
    }
    let msg_vec = msg_vec.make_contiguous();
    let mut display = DisplayChunks::new();
    // ------------------
    // Display on 5
    display.append_many(&msg_vec[..5]);
    display.hide_older();
    assert_eq!(display.status, DisStatus::AllVisible);
    // Display on 15
    display.append_many(&msg_vec[5..15]);
    display.hide_older();
    assert_eq!(display.status, DisStatus::AllVisible);
    // Display on 19
    display.append_many(&msg_vec[15..19]);
    display.hide_older();
    assert_eq!(display.status, DisStatus::AllVisible);
    // Display on 20
    display.append_new(msg_vec[19].clone());
    let hidden = msg_vec[19].ulid();
    display.hide_older();
    assert_eq!(display.status, DisStatus::PartiallyHidden(19, hidden));
    // Display on 21
    display.append_many(&msg_vec[20..21]);
    display.hide_older();
    assert_eq!(display.status, DisStatus::PartiallyHidden(19, hidden));
    // Display on 40
    display.append_many(&msg_vec[21..]);
    display.hide_older();
    assert_eq!(display.status, DisStatus::PartiallyHidden(19, hidden));
}

#[test]
fn display_append_older_chunk_test() {
    let act_room = Id::new(Tb::Room);
    let acc = crate::cont::acc::Account {
        acc_id: Id::new(Tb::Acc),
        username: "Karol".into(),
        av: std::rc::Rc::new(vec![]),
    };
    let mut chunks = RoomMsgChunks::new(act_room.clone());
    let mut msg_vec = VecDeque::new();
    for _ in 0..62 {
        std::thread::sleep(std::time::Duration::from_millis(2));
        let msg = MsgViewData::new_from_click(act_room.clone(), &acc);
        msg_vec.push_back(msg);
    }
    let msg_vec = msg_vec.make_contiguous();
    let mut display = DisplayChunks::new();
    let mut display_alt = DisplayChunks::new();
    display.append_many(&msg_vec[40..]);
    display_alt.append_many(&msg_vec[40..]);
    // ------------------
    // Display on 42
    display.append_older_chunk(&msg_vec[20..40]);
    display_alt.append_older_chunk_alt(&msg_vec[20..40]);
    assert_eq!(display.total_stored, 42);
    assert_eq!(display.vec.front().unwrap().ulid(), msg_vec[20].ulid());
    assert_eq!(display_alt.total_stored, 42);
    assert_eq!(display_alt.vec.front().unwrap().ulid(), msg_vec[20].ulid());
    // Display on 62
    display.append_older_chunk(&msg_vec[..20]);
    display_alt.append_older_chunk_alt(&msg_vec[..20]);
    assert_eq!(display.total_stored, 62);
    assert_eq!(display.vec.front().unwrap().ulid(), msg_vec[0].ulid());
    assert_eq!(display_alt.total_stored, 62);
    assert_eq!(display_alt.vec.front().unwrap().ulid(), msg_vec[0].ulid());
}