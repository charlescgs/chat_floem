use std::cell::RefCell;
use std::rc::Rc;

use floem::prelude::*;
use floem::reactive::{use_context, Trigger};
use floem::taffy::{AlignItems, FlexDirection};
use tracing_lite::{debug, info, trace};
use im::{vector, Vector};

use crate::{ChatState, MsgView};
use super::chunks::RoomMsgChunks;



pub fn main_msg_view(msgs: Rc<RefCell<RoomMsgChunks>>) -> impl IntoView {
    info!("->> main_msg_view");
    let state = use_context::<Rc<ChatState>>().unwrap();
    let msg_view = use_context::<RwSignal<MsgView>>().unwrap();
    let new_msg_scroll_end = Trigger::new();
    
    let room_msgs = move || {
        debug!("Computing room_msgs..");
        match msg_view.get() {
            MsgView::None => vector!(),
            MsgView::NewMsg(room) => {
                let mut chunk_iter = vector!();
                for each in msgs.borrow().reload() {
                    chunk_iter.append(each.msgs.clone());
                }
                trace!("room_msgs: reloaded msgs({})", chunk_iter.len());
                for m in &chunk_iter {
                    println!("{} - {}", m.msg.created.human_formatted(), m.msg.text.current);
                }
                
                new_msg_scroll_end.notify();
                chunk_iter
            },
            MsgView::LoadMore(rect) => {
                let mut chunk_iter = vector!();
                for each in msgs.borrow_mut().load_next() {
                    chunk_iter.append(each.msgs.clone());
                }
                for m in &chunk_iter {
                    println!("{} - {}", m.msg.created.human_formatted(), m.msg.text.current);
                }
                trace!("room_msgs: loaded more msgs({})", chunk_iter.len());
                chunk_iter
            }
        }
    };

    dyn_stack(
        move || {
            // debug!("->> msg fn");
            let chunks = room_msgs();
            chunks.into_iter().rev().enumerate()
        },
        |(idx, _)| *idx,
        |(_, msg)| {
            // trace!("dyn_stack: msg (view_fn)");
            let id = msg.id.id.0;
            // let _is_owner = msg.room_owner;
            msg
                .style(move |s| s.apply_if(id % 2 == 0, // for now
                    // is_owner,
                    |s| s.align_self(AlignItems::End)
                    )
                )
            }
    // virtual_stack(
    //     VirtualDirection::Vertical,
    //     VirtualItemSize::Fixed(Box::new(|| 70.) ),
    //     move || {
    //         debug!("v_stack: each_fn");
    //         room_msgs().into_iter().rev().enumerate().collect::<Vector<_>>()
    //     },
    //     |(idx, msg)| {
    //         debug!("v_stack: key_fn");
    //         *idx
    //     },
    //     |(_, msg)| {
    //         debug!("v_stack: view_fn");
    //         let id = msg.id.id.0;
    //         msg
    //             .style(move |s| s.apply_if(id % 2 == 0, // for now
    //                 |s| s.align_self(AlignItems::End)
    //                 )
    //             )
    //         }
    ).debug_name("msgs list")
    .style(|s| s
        .flex_direction(FlexDirection::ColumnReverse)
        .width_full()
        .align_items(AlignItems::Start)
        .column_gap(5.)
        // .size_full()
    )
    .scroll()
    .debug_name("msgs scroll")
    .style(|s| s
        .size_full()
        .padding(5.)
        .padding_right(7.)
    )
    .scroll_style(|s| s
        .handle_thickness(6.)
        .shrink_to_fit()
    )
    .scroll_to_percent(move || {
        trace!("scroll_to_percent");
        new_msg_scroll_end.track();
        100.0
    })
    .on_scroll(move |rect| {
        if rect.y0 == 0.0 {
            debug!("on_scroll: load_more notified!");
            msg_view.set(MsgView::LoadMore(rect));
            // load_more.notify();
        }
    })
}