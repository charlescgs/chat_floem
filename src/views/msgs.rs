use std::cell::RefCell;

use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::{batch, create_effect, create_memo, create_updater, use_context, Trigger};
use floem::taffy::prelude::TaffyGridLine;
use floem::taffy::{AlignItems, FlexDirection, GridPlacement, Line};
use floem::views::{dyn_stack, stack, tab, Decorators, ScrollExt};
use tracing_lite::{debug, info, trace, warn};
use im::Vector;
use ulid::Ulid;

use crate::view_data::msg::MsgViewData;
use crate::view_data::session::APP;
use crate::view_data::MsgEvent;



#[derive(Clone, Debug)]
pub enum RoomMsgUpt {
    NoUpdate,
    New,
    Changed(Ulid),
    Deleted(Ulid)
}


/// This function:
/// - [ ] organize msgs,
/// - [ ] paint msgs,
/// - [ ] update view on changes
/// - [ ] communicate with rooms
/// - [ ] communicate with backend
pub fn msgs_view_v2() -> impl View {
    info!("->> msgs_view");
    // -- Needed elements
    let msg_event = use_context::<RwSignal<MsgEvent>>().unwrap();
    let rooms = APP.with(|app| app.rooms);
    let active_room = APP.with(|app| app.active_room);
    let rooms_tabs = APP.with(|app| app.rooms_tabs);
    let scroll_to_end = Trigger::new();

    // -- Effect and derives needed for the view

    // High level effect; matches on the update type and sends notif to specific room upt fn
    create_effect(move |_| {
        debug!("== effect(msg_event): | inside msgs_view fn|");
        match msg_event.get() {
            MsgEvent::None => {
                trace!("effect: | msgs_view | msg event: None");
                // No event; just return previous state
                // msg_upt.set(MsgUpt::NoUpdate);
            },
            MsgEvent::NewFor(room) => {
                trace!("effect: | msgs_view | msg event: NewFor({room})");
                // New msg; just append it onto existing vec
                if let Some(tab) = rooms_tabs.with_untracked(|rt| rt.get(&room).cloned()) {
                    trace!("effect: | msgs_view | roomsTabs is `Some`");
                    tab.2.set(RoomMsgUpt::New);
                }
            },
            MsgEvent::UpdatedFor { room, msg } => {
                trace!("effect: | msgs_view | msg event: UpdatedFor({room}: {msg})");
                // Updated msg, just search and replace it
                if let Some(tab) = rooms_tabs.with_untracked(|rt| rt.get(&room).cloned()) {
                    tab.2.set(RoomMsgUpt::Changed(msg));
                }
            },
            MsgEvent::Deleted { room, msg } => {
                trace!("effect: | msgs_view | msg event: Deleted({room}: {msg})");
                // Deleted msg, just remove it
                if let Some(tab) = rooms_tabs.with_untracked(|rt| rt.get(&room).cloned()) {
                    tab.2.set(RoomMsgUpt::Deleted(msg));
                }
            },
        } 
    });

    // let room_msgs = create_memo(move |_| -> Vector<MsgViewData> {
    //     let mut msgs = Vector::new();
    //     // let room_msgs = move || -> Vector<MsgViewData> {
    //     match msg_upt.get() {
    //         MsgUpt::NoUpdate => msgs,
    //         MsgUpt::New(ref idx) => {
    //             rooms.with_untracked(|r| {
    //                 match r.get(idx) {
    //                     Some(room) => {
    //                         let Some(new_msg) = room.last_msg.get_untracked() else {
    //                             return msgs
    //                         };
    //                         msgs.push_back(new_msg);
    //                         msgs
    //                     },
    //                     None => msgs
    //                 }
    //             })
    //         },
    //         MsgUpt::Changed(ulid) => todo!(),
    //         MsgUpt::Deleted(ulid) => todo!(),
    //     }
    // });

    
    // -- View stack
    stack((
        tab(move || {
            match active_room.get() {
                Some(id) => {
                    trace!("tab: active_fn: Some({})", id.idx);
                    rooms_tabs.with_untracked(|rt| rt.get(&id.id).unwrap().0)
                },
                None => {
                    trace!("tab: active_fn: None = 0");
                    0
                }
            }},
            move || {
                trace!("tab: each_fn");
                rooms.get()
            },
            |(idx, _)| {
                trace!("tab: key_fn: {idx}");
                *idx
            },
            move |(idx, room)| {
                let get_upt = room.get_update;
                let mut room_chunks = room.msgs;
                // let x = create_updater(compute, on_change);
                let mut msgs_vec = RwSignal::new(Vector::new());
                room_chunks.with_untracked(|c|
                    if c.total_msgs != 0 {
                        batch(|| {
                            for each in c.reload() {
                                msgs_vec.update(|v| v.append(each.msgs.clone()));
                            }
                        });
                    }
                );
                create_effect(move |_| {
                    debug!("== memo: msgs tab({idx})");
                    match get_upt.get() {
                        RoomMsgUpt::NoUpdate => {},
                        RoomMsgUpt::New => {
                            if let Some(new_msg) = room.last_msg.get_untracked() {
                                debug!("RoomMsgUpt: {idx} with new msg: {}", new_msg.id.id);
                                msgs_vec.update(|v| v.push_back(new_msg));
                                println!("msgs vector len: {}", msgs_vec.with_untracked(|v| v.len()));
                            } else {
                                warn!("RoomMsgUpt: {idx} last msg fn returned None")
                            }
                            // msgs
                        },
                        RoomMsgUpt::Changed(msg_id) => {
                            if let Some(changed_msg) = room_chunks.with_untracked(|rc| rc.find_msg(msg_id).cloned()) {
                                let Some(idx) = msgs_vec.with_untracked(|v| v.index_of(&changed_msg)) else { return };
                                debug!("RoomMsgUpt: {idx} with upt msg: {}", changed_msg.id.id);
                                msgs_vec.update(|v| { v.update(idx, changed_msg); });
                            }
                            // msgs
                        },
                        RoomMsgUpt::Deleted(ref msg_id) => {
                            let mut del_idx = 0;
                            if let Some(del_msg) = msgs_vec.with_untracked(|v| v.iter().find(|m| &m.id.id == msg_id).cloned()) {
                                let del_idx_found =
                                    if let Some(idx) = msgs_vec.with_untracked(|v| v.index_of(&del_msg)) {
                                    del_idx = idx;
                                    debug!("RoomMsgUpt: {idx} with {}", del_msg.id.id);
                                    true
                                    } else {
                                        false
                                    };
                                if del_idx_found {
                                    msgs_vec.update(|v| { v.remove(del_idx); });
                                }
                            }
                            // msgs
                        }
                    }
                });
                
                dyn_stack(
                    move || {
                        let chunks = msgs_vec.get();
                        info!("->> dyn_stack: msg(each_fn) (with {} msg/s)", chunks.len());
                        for each in chunks.iter() {
                            println!("{}", each.id)
                        }
                        chunks.into_iter().enumerate()
                    },
                    |(idx, msg)| {
                        info!("dyn_stack: msg(key_fn) for {}", msg.id.id);
                        *idx
                    },
                    |(_, msg)| {
                        trace!("dyn_stack: msg(view_fn): {}", msg.id);
                        let id = msg.id.id.0;
                        // let _is_owner = msg.room_owner;
                        msg
                            .style(move |s| s.apply_if(id % 2 == 0, // for now
                                // is_owner,
                                |s| s.align_self(AlignItems::End)
                                )
                            )
                        }
                    ).debug_name("msgs list")
                    .style(|s| s
                        .flex_direction(FlexDirection::Column)
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
                    // .scroll_to_percent(move || {
                    //     trace!("scroll_to_percent");
                    //     new_msg_scroll_end.track();
                    //     100.0
                    // })
                    // .on_scroll(move |rect| {
                    //     if rect.y0 == 0.0 {
                    //         debug!("on_scroll: load_more notified!");
                    //         msg_view.set(MsgView::LoadMore(rect));
                    //         load_more.notify();
                    //     }
                    // })
            }
        ).debug_name("msgs tabs")
        .style(|s| s.size_full())
        // .on_resize(move |_rect| {
            // scroll_pos.set(rect);
        // })
        ,
    )).debug_name("msgs stack")
    .style(|s| s
        .padding(5.)
        .background(Color::LIGHT_GREEN)
        .border_color(Color::BLACK)
        .border(1.)
        .grid_column(Line {
            start: GridPlacement::from_line_index(2),
            end: GridPlacement::Span(2)
        })
        .grid_row(Line {
            start: GridPlacement::from_line_index(2),
            end: GridPlacement::Span(2)
        })
    )
}