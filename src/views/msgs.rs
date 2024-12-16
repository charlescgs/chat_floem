use std::collections::BTreeMap;
use std::time::Duration;

use floem::kurbo::Point;
use floem::peniko::Color;
use floem::prelude::*;
use floem::reactive::{create_effect, use_context, Trigger};
use floem::taffy::prelude::TaffyGridLine;
use floem::taffy::{AlignItems, FlexDirection, GridPlacement, Line};
use floem::views::{dyn_stack, stack, tab, Decorators, ScrollExt};
use tracing_lite::{debug, info, trace, warn};
use im::Vector;
use ulid::Ulid;

use crate::view_data::session::APP;
use crate::view_data::{run_on_second_trigger, MsgEvent};



#[derive(Clone, Debug)]
pub enum RoomMsgUpt {
    NoUpdate,
    NewMany,
    New,
    Changed(Ulid),
    Deleted(Ulid)
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
                trace!("tab: each_fn");
                rooms.get()
            },
            |(idx, _)| {
                trace!("tab: key_fn: {idx}");
                *idx
            },
            move |(idx, room)| {
                let scroll_to_end = Trigger::new();
                // -- Tab logic and state
                let get_upt = room.get_update;
                // -- Room messages
                let room_chunks = room.msgs;
                // let x = create_updater(compute, on_change);
                let msgs_count = room.msgs_count;
                // let cx = APP.with(|app| app.provide_scope());
                let msgs_btree = RwSignal::new(BTreeMap::new());
                let scroll_rect = RwSignal::new(Point::ZERO);
                let is_active = room.is_active;
                let load_more = Trigger::new();
                let reload = Trigger::new();
                let room_idx = room.idx();
                // -- Tracks how many chunks is in this room

                create_effect(move |_| {
                    trace!("== effect: scroll end on tab switch for {room_idx}");
                    is_active.with(|cell| {
                        if cell.get() == true {
                            scroll_to_end.notify();
                        }
                    });
                });

                create_effect(move |_| {
                    run_on_second_trigger(load_more, move || {
                        room_chunks.with_untracked(|chunks| {
                            debug!("== effect(with debounce): load_more msgs");
                            // -- Only load next chunk if more left
                            let count = msgs_count.get_untracked();
                            let total = chunks.total_msgs;
                            println!("if {count} == {total} then load next chunk");
                            // Case 1: Everything loaded
                            // Case 2: Nothing loaded, load next
                            // Case 3: One loaded, load next
                            // Case 4: Many loaded, load next
                            // Case 5: 
                            // 1. Get data from room chunks and display chunks
                            let total_chunks = chunks.chunks_count;
                            let last_on_display = chunks.oldest_display_chunk_idx.get(); //  need - 1
                            // 2. Compare them and act upon the result:
                            // assert!(dis_chunks.total == total_chunks);
                            
                            let older_chunk = chunks.load_older_chunk();
                            println!("loaded chunk len: {}", older_chunk.len());
                            if !older_chunk.is_empty() {
                                trace!("== effect(with debounce): loaded next chunk");
                                msgs_btree.update(|btree| {
                                    for msg in older_chunk {
                                        btree.insert(msg.id.id, msg.clone());
                                    }
                                });
                            }
                        });
                    });
                });

                create_effect(move |_| {
                    debug!("== effect: msgs tab({idx})");
                    match get_upt.get() {
                        RoomMsgUpt::NoUpdate => {},
                        RoomMsgUpt::NewMany => {
                            msgs_btree.update(|btree| {
                                room_chunks.with_untracked(|chunks| {
                                    debug!("RoomMsgUpt::NewMany: {idx} with new msgs, loading new content");
                                    let current_youngest_msg = btree.keys().last().cloned();
                                    btree.extend(chunks.load_new_content(current_youngest_msg).into_iter().map(|m| (m.id.id, m)));
                                })
                            });
                        },
                        RoomMsgUpt::New => {
                            if let Some(new_msg) = room.last_msg.get_untracked() {
                                debug!("RoomMsgUpt::New: {idx} with new msg: {}", new_msg.id.id);
                                msgs_btree.update(|v| { v.insert(new_msg.id.id, new_msg); });
                                println!("msgs vector len: {}", msgs_btree.with_untracked(|btree| btree.len()));
                                scroll_to_end.notify();
                            } else {
                                warn!("RoomMsgUpt: {idx} last msg fn returned None")
                            }
                            // msgs
                        },
                        RoomMsgUpt::Changed(msg_id) => {
                            if let Some(changed_msg) = room_chunks.with_untracked(|rc| rc.find_msg(msg_id).cloned()) {
                                debug!("RoomMsgUpt::Changed: {idx} with upt msg: {}", changed_msg.id.id);
                                msgs_btree.update(|btree|
                                    if let Some(msg) = btree.get_mut(&changed_msg.id.id) {
                                        *msg = changed_msg;
                                    }
                                );
                            }
                        },
                        RoomMsgUpt::Deleted(ref msg_id) => {
                            debug!("RoomMsgUpt::Deleted: {idx} with {msg_id}");
                            msgs_btree.update(|btree| { btree.remove(&msg_id); });
                        }
                    }
                });
                
                dyn_stack(
                    move || {
                        let chunks = msgs_btree.get();
                        info!("->> dyn_stack: msg(each_fn) (with {} msg/s)", chunks.len());
                        for (each_id, _) in chunks.iter() {
                            println!("{each_id}")
                        }
                        chunks.into_iter()
                    },
                    |(idx, msg)| {
                        info!("dyn_stack: msg(key_fn) for {}", msg.id.id);
                        *idx
                    },
                    |(_, msg)| {
                        trace!("dyn_stack: msg(view_fn): {}", msg.id);
                        let id = msg.id.id.0;
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
                    // .ensure_visible(move || {})
                    .on_scroll(move |rect| {
                        // println!("{:?}", rect.origin());
                        // println!("{}", Point::new(rect.x0, rect.y0));
                        if msgs_count.get() > 20 {
                            // scroll_rect.set(Point::new(rect.x0, rect.y0));
                            if rect.y0 == 0.0 {
                                trace!("dyn_stack: msg: on_scroll: load_more notified!");
                                load_more.notify();
                            }
                        }
                    })
                    .scroll_to_percent(move || {
                        scroll_to_end.track();
                        trace!("scroll_to_end notified for {}", room.idx());
                        100.0
                    })
                    // .scroll_to(move || {
                    //     Some(scroll_rect.get())
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