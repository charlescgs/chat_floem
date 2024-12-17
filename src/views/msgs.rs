use std::collections::BTreeMap;
use std::rc::Rc;

use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::{create_effect, use_context, Trigger};
use floem::taffy::prelude::TaffyGridLine;
use floem::taffy::{AlignItems, FlexDirection, GridPlacement, Line};
use floem::views::{dyn_stack, stack, tab, Decorators, ScrollExt};
use im::Vector;
use tracing_lite::{debug, info, trace, warn};
use ulid::Ulid;

use crate::view_data::session::APP;
use crate::view_data::MsgEvent;



#[derive(Clone, Debug)]
pub enum RoomMsgUpt {
    New,
    NewMany,
    NoUpdate,
    Changed(Ulid),
    Deleted(Ulid),
    LoadMore
}


/// This function:
/// - [x] organize msgs,
/// - [x] paint msgs,
/// - [x] update view on changes
/// - [ ] communicate with rooms
/// - [ ] communicate with backend
pub fn msgs_view_v2() -> impl View {
    info!("->> msgs_view");
    // -- Needed elements
    let msg_event = use_context::<RwSignal<MsgEvent>>().unwrap();
    let rooms = APP.with(|app| app.rooms);
    let active_room = APP.with(|app| app.active_room);
    let rooms_tabs = APP.with(|app| app.rooms_tabs);
    let show_load_more_button = use_context::<RwSignal<bool>>().unwrap();

    // -- Effect and derives needed for the view

    // High level effect; matches on the update type and sends notif to specific room upt fn
    create_effect(move |_| {
        debug!("== effect(msg_event): | inside msgs_view fn|");
        match msg_event.get() {
            MsgEvent::None => {
                trace!("effect: | msgs_view | msg event: None");
                // No event; just return previous state
            },
            MsgEvent::NewFor(room) => {
                trace!("effect: | msgs_view | msg event: NewFor({room})");
                // New msg; just append it onto existing vec
                if let Some(tab) = rooms_tabs.with_untracked(|rt| rt.get(&room).cloned()) {
                    trace!("effect: | msgs_view | roomsTabs is `Some`");
                    tab.2.set(RoomMsgUpt::New);
                }
            },
            MsgEvent::NewManyFor(room) => {
                trace!("effect: | msgs_view | msg event: NewManyFor({room})");
                if let Some(tab) = rooms_tabs.with_untracked(|rt| rt.get(&room).cloned()) {
                    trace!("effect: | msgs_view | roomsTabs is `Some`");
                    tab.2.set(RoomMsgUpt::NewMany);
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
            }
        } 
    });
    // -- View stack
    stack((
        tab(move || {
            match active_room.get() {
                Some(id) => {
                    trace!("tab: active_fn: Some({})", id.idx);
                    id.idx
                },
                None => {
                    trace!("tab: active_fn: None = 0");
                    0
                }
            }},
            move || {
                trace!("tab: each_fn"); // FIXME: called twice during startup
                rooms.get()
            },
            |(idx, _)| {
                trace!("tab: key_fn: {idx}");
                *idx
            },
// MARK: tab
            move |(idx, room)| {
                let scroll_to_end = Trigger::new();
                // -- Tab logic and state
                // let cx = APP.with(|app| app.provide_scope());
                let this_room = Rc::new(room);
                let get_upt = this_room.get_update;
                // -- Room messages
                let room_chunks = this_room.msgs;
                // let msgs_count = this_room.msgs_count;
                let msgs_vec = RwSignal::new(Vector::new());
                
                // let msgs_vec = RwSignal::new(Vector::new());
                let is_active = this_room.is_active;
                // let load_more = Trigger::new();
                let room_idx = this_room.idx();
                // -- Tracks how many chunks is in this room
                
                let room = this_room.clone();
                create_effect(move |_| {
                    trace!("== effect: scroll end on tab switch for {room_idx}");
                    is_active.with(|cell| {
                        // 
                        if cell.get() == true {
                            // if let Some(id) = room_chunks.with_untracked(|rc| {
                            //     // rc.
                            // }) {
                            //     msgs_vec.update(|mv| {

                            //         // bt.split_off(&room.clone().get_msg_range_to_display().start);
                            //     });
                            // }
                            scroll_to_end.notify();
                        }
                    });
                });

                let room = this_room.clone();
                create_effect(move |_| {
                    debug!("== effect: msgs tab({idx})");
                    match get_upt.get() {
                        RoomMsgUpt::NoUpdate => {},
                        RoomMsgUpt::LoadMore => {
                            debug!("RoomMsgUpt::LoadMore");
                            if room.msgs_count.get() > msgs_vec.with_untracked(|mv| mv.len() as u16) {
                                msgs_vec.update(|mv| {
                                    room_chunks.with_untracked(|chunks| {
                                        for each in chunks.load_older_chunk() {
                                            mv.push_back(each.clone());
                                        }
                                    })
                                });
                            }
                        },
                        RoomMsgUpt::NewMany => {
                            msgs_vec.update(|mv| {
                                room_chunks.with_untracked(|chunks| {
                                    debug!("RoomMsgUpt::NewMany: {idx} with new msgs, loading new content");
                                    let current_youngest_msg = mv.last().map(|m| m.id.id); // FIXME!
                                    mv.extend(
                                        chunks
                                            .load_new_content(current_youngest_msg, true)
                                            .into_iter()
                                    );
                                })
                            });
                        },
                        RoomMsgUpt::New => {
                            if let Some(new_msg) = room.msgs.with_untracked(|chunks| chunks.last_msg().cloned()) {
                                debug!("RoomMsgUpt::New: {idx} with new msg: {}", new_msg.id.id);
                                msgs_vec.update(|v| v.push_back(new_msg));
                                println!("msgs vector len: {}", msgs_vec.with_untracked(|mv| mv.len()));
                                scroll_to_end.notify();
                            } else {
                                warn!("RoomMsgUpt: {idx} last msg fn returned None")
                            }
                        },
                        RoomMsgUpt::Changed(msg_id) => {
                            if let Some(changed_msg) = room_chunks.with_untracked(|rc| rc.find_msg(msg_id).cloned()) {
                                debug!("RoomMsgUpt::Changed: {idx} with upt msg: {}", changed_msg.id.id);
                                let Some(idx) = msgs_vec.with_untracked(|v| v.index_of(&changed_msg)) else { return };
                                msgs_vec.update(|mv|
                                    if let Some(msg) = mv.get_mut(idx) {
                                        *msg = changed_msg;
                                    }
                                );
                            }
                        },
                        RoomMsgUpt::Deleted(ref msg_id) => {
                            debug!("RoomMsgUpt::Deleted: {idx} with {msg_id}");
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
                        }
                    }
                });
                
                dyn_stack(
                    move || {
                        let chunks = msgs_vec.get(); // FIXME: called twice during new msg
                        info!("->> dyn_stack: msg(each_fn) (with {} msg/s)", chunks.len());
                        // for (each_id, _) in chunks.iter() {
                        //     println!("{each_id}")
                        // }
                        chunks.into_iter().enumerate()
                        
                    },
                    |(idx, msg)| {
                        info!("dyn_stack: msg(key_fn) for {}", msg.id.id);
                        *idx
                    },
                    |(_, msg)| {
                        trace!("dyn_stack: msg(view_fn): {}", msg.id);
                        let is_owner = msg.room_owner;
                        msg.style(move |s| s.apply_if(is_owner,
                            |s| s.align_self(AlignItems::End)
                            ))
                        }
                    ).debug_name("msgs list")
                    .style(|s| s
                        .flex_direction(FlexDirection::Column)
                        .width_full()
                        .align_items(AlignItems::Start)
                        .column_gap(5.)
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
                        .propagate_pointer_wheel(true)
                    )
                    .on_scroll(move |rect| {
                        if rect.y0 == 0.0 {
                            println!("{:?}", rect.origin());
                            if this_room.msgs_count.get() > 20 {
                                // println!("on_scroll: load_more true!");
                                show_load_more_button.set(true);
                                // load_more.notify();
                            }
                        } else {
                            // println!("on_scroll: load_more false!");
                            show_load_more_button.set(false);
                        }
                    })
                    .scroll_to_percent(move || {
                        scroll_to_end.track();
                        trace!("scroll_to_end notified for {}", room_idx);
                        100.0
                    })
        }).debug_name("msgs tabs")
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