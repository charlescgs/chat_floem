use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;
use tracing_lite::{trace, Subscriber};

use crate::chunks::chunks::RoomMsgChunks;
use crate::cont::acc::Account;
use crate::util::{Id, Tb};
use crate::view_data::msg::MsgViewData;

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
    let from_2_case_res = chunks.load_new_content(Some(from_2_case), false);
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
    let from_3_case_res = chunks.load_new_content(Some(from_3_case), false);
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
    let from_19_case_res = chunks.load_new_content(Some(from_19_case), false);
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
    let from_20_case_res = chunks.load_new_content(Some(from_20_case), false);
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
    let from_21_case_res = chunks.load_new_content(Some(from_21_case), false);
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
    let from_50_case_res = chunks.load_new_content(Some(from_50_case), false);
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
    let no_never_msg_case_res = chunks.load_new_content(Some(no_never_msg_case), false);
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