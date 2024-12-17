use std::time::Duration;

use chat_util::gen::gen_u64_in_range;
use floem::prelude::*;
use floem::menu::{Menu, MenuItem};
use floem::reactive::{batch, create_memo};
use floem::reactive::create_effect;
use floem::reactive::use_context;
use floem::taffy::prelude::TaffyGridLine;
use floem::taffy::{AlignContent, GridPlacement, Line};
use tracing_lite::{debug, error, info, trace};
use ulid::Ulid;

use crate::cont::acc::Account;
use crate::view_data::msg::MsgViewData;
use crate::view_data::room::RoomViewData;
use crate::view_data::session::APP;
use crate::view_data::MsgEvent;

use super::msgs::RoomMsgUpt;


#[derive(Clone, Debug)]
pub enum EditList {
    None,
    Room,
    Msg,
    Account
}

#[derive(Clone, Debug)]
pub enum NewList {
    None,
    Room,
    Msg,
    Msgs,
    Account
}

impl std::fmt::Display for EditList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditList::None => f.write_str("None"),
            EditList::Room => f.write_str("Room"),
            EditList::Msg => f.write_str("Msg"),
            EditList::Account => f.write_str("Account"),
        }
    }
}


/// - [ ] reacts on clicked buttons/menus with actions.
pub fn toolbar_view_v2() -> impl IntoView {
    let edit_list_signal = RwSignal::new(EditList::None);
    let new_list_signal = RwSignal::new(NewList::None);
    // -- Id is a room, that got an update
    let msg_event = use_context::<RwSignal<MsgEvent>>().unwrap();
    let new_room_editor_doc = use_context::<RwSignal<Option<Ulid>>>().unwrap();
    let show_load_more_button = use_context::<RwSignal<bool>>().unwrap();
    let show_load_memo = create_memo(move |_| show_load_more_button.get());
    // -- Action to create test room on click
    create_effect(move |_| {
        let new = new_list_signal.get();
        trace!("->> effect for `New Menu`: {new:#?}");
        match new {
            NewList::None => { trace!("Clicked NewList::None"); },
            NewList::Room => {
                let room_view = RoomViewData::new_from_click();
                trace!("Clicked NewList::Room - idx: {} | room: {}", room_view.idx(), room_view.room_id.ulid());
                
                APP.with(|app| {
                    batch(|| {
                        app.accounts.update(|accs| { accs.insert(room_view.owner.acc_id.id, room_view.owner.clone());} );
                        app.rooms.update(|rooms| {
                            debug!("room: app.rooms.update");
                            if let Some(ret) = rooms.insert(room_view.idx(), room_view.clone()) {
                                error!("value returned when attempted to insert {}", ret.room_id)
                            }
                        });
                        app.rooms_tabs.update(|tabs| {
                            tabs.insert(
                                room_view.room_id.id,
                                (room_view.idx(), room_view.view_id, room_view.get_update)
                            );
                        });
                    });
                    new_room_editor_doc.set(Some(room_view.room_id.id));
                    trace!("Created and inserted test RoomViewData")
                });
            },
            NewList::Msg => {
                trace!("Clicked NewList::Msg");
                // -- Get active room
                if let Some(active) = APP.with(|app| app.active_room.get_untracked()) {
                    APP.with(|app| {
                        app.rooms.with_untracked(|rooms| {
                            // -- Get or create account (only in early faze)
                            let acc = {
                                let room = rooms.get(&active.idx).unwrap();
                                let rand = gen_u64_in_range(0..3);
                                if rand == 0 {
                                    room.owner.clone()
                                } else {
                                    let keys_vec = room.members.keys().cloned().collect::<Vec<_>>();
                                    println!("key_vec len: {}", keys_vec.len());
                                    let key = keys_vec.get(rand as usize - 1).cloned().unwrap();
                                    room.members.get(&key).unwrap().clone()
                                }
                            };
                            // -- Create msg
                            let msg = MsgViewData::new_from_click(active.id(), &acc);
                            // -- Append msg onto `msgs` and `last_msg`
                            if let Some(room) = rooms.get(&active.idx) {
                                room.msgs.update(|msgs| {
                                    msgs.append_new_msg(msg.clone());
                                    info!("New msg appended!")
                                });
                                let old = room.msgs_count.get();
                                room.msgs_count.set(old + 1);

                                room.last_msg.set(Some(msg));
                                // -- Notify subscribers about new msg event
                                msg_event.set(MsgEvent::NewFor(room.room_id.id));
                                // msg_event
                            }
                        })
                    });
                }
            },
            NewList::Msgs => {
                trace!("Clicked NewList::Msgs");
                // -- Get active room
                if let Some(active) = APP.with(|app| app.active_room.get_untracked()) {
                    APP.with(|app| {
                        app.rooms.with_untracked(|rooms| {
                            // -- Get or create account (only in early faze)
                            let acc = {
                                let room = rooms.get(&active.idx).unwrap();
                                let rand = gen_u64_in_range(0..3);
                                if rand == 0 {
                                    room.owner.clone()
                                } else {
                                    let keys_vec = room.members.keys().cloned().collect::<Vec<_>>();
                                    println!("key_vec len: {}", keys_vec.len());
                                    let key = keys_vec.get(rand as usize - 1).cloned().unwrap();
                                    room.members.get(&key).unwrap().clone()
                                }
                            };
                            // -- Create msgs
                            let mut msgs = vec!();
                            for _ in 0..40 {
                                std::thread::sleep(Duration::from_millis(2));
                                msgs.push(MsgViewData::new_from_click(active.id(), &acc))
                            }
                            let last_msg = msgs.last().unwrap().clone();
                            // -- Append msg onto `msgs` and `last_msg`
                            if let Some(room) = rooms.get(&active.idx) {
                                room.msgs.update(|chunks| {
                                    for msg in msgs {
                                        chunks.append_new_msg(msg);
                                    }
                                });
                                info!("{} new msgs appended!", room.msgs_count.get());
                                room.msgs_count.set(room.msgs.with_untracked(|m| m.total_msgs));
                                room.last_msg.set(Some(last_msg));
                                // -- Notify subscribers about new msg event
                                msg_event.set(MsgEvent::NewManyFor(room.room_id.id));
                            }
                        })
                    });
                }
            },
            NewList::Account => {
                trace!("Clicked NewList::Account");
                if let Some(acc) = Account::new_from_click() {
                    APP.with(|app| {
                        app.accounts.update(|accs| { accs.insert(acc.acc_id.id.clone(), acc); })
                    })
                }
            }
        }
    });
    
    let new_menu = "New".button().popout_menu(move || {
        Menu::new("")
            .entry(MenuItem::new("Msg").action(move || {
                new_list_signal.set(NewList::Msg);
            }))
            .entry(MenuItem::new("Msgs").action(move || {
                new_list_signal.set(NewList::Msgs);
            }))
            .entry(MenuItem::new("Room").action(move || {
                new_list_signal.set(NewList::Room);
            }))
            .separator()
            .entry(MenuItem::new("Account").action(move || {
                new_list_signal.set(NewList::Account);
            }))
    });

    let edit_menu = "Edit".button().popout_menu(move || {
        Menu::new("")
            .entry(MenuItem::new("Account").action(move || {
                edit_list_signal.set(EditList::Account);
            }))
            .separator()
            .entry(MenuItem::new("Room").action(move || {
                edit_list_signal.set(EditList::Room);
            }))
            .entry(MenuItem::new("Msg").action(move || {
                edit_list_signal.set(EditList::Msg);
            }))
    });
    
    stack((
        h_stack((
            stack((
                new_menu,
                edit_menu,
                "Settings".button().action(move || {}),
                "About".button().action(move || {}),
            )).style(|s| s
                .padding(5.)
                .row_gap(5.)
            ),
            "load more".button().action(move || {
                let act_room = APP.with(|app| {
                    if let Some(ar) = app.active_room.get_untracked() {
                        app.rooms_tabs.with_untracked(|rt| {
                            if let Some(act_tab) = rt.get(&ar.id) {
                                act_tab.2.set(RoomMsgUpt::LoadMore);
                            }
                        });
                    }
                }); 
            })
                .disabled(move || !show_load_memo.get())
                .style(|s| s.disabled(|s| s
                    .background(Color::TRANSPARENT)
                    .color(Color::TRANSPARENT)
                ))
                .container()
                .style(|s| s.padding(5.))
        )).style(|s| s
            .size_full()
            .justify_content(AlignContent::SpaceBetween)
        ),
    )).debug_name("menu toolbar")
    .style(|s| s
        .background(Color::MEDIUM_ORCHID)
        .border_color(Color::BLACK)
        .border(1.)
        .grid_column(Line {
            start: GridPlacement::from_line_index(1),
            end: GridPlacement::Span(3)
        })
        .grid_row(Line {
            start: GridPlacement::from_line_index(1),
            end: GridPlacement::Span(1)
        })
    )
}