use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::thread::sleep;
use std::time::Duration;

use floem::prelude::*;
use floem::menu::{Menu, MenuItem};
use floem::reactive::batch;
use floem::reactive::create_effect;
use floem::reactive::use_context;
use floem::taffy::prelude::TaffyGridLine;
use floem::taffy::{AlignContent, GridPlacement, Line};
use tracing_lite::{error, trace};

use crate::cont::acc::Account;
use crate::view_data::room::RoomViewData;
use crate::view_data::session::APP;
use crate::view_data::MsgEvent;
use crate::views::chunks::RoomMsgChunks;
use crate::views::msg::MsgCtx;
use crate::views::room::{RoomCtx, ROOM_IDX};
use crate::{ChatState, MsgView};


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
    // let msgs_tracker = use_context::<RwSignal<Option<Id>>>().unwrap();
    // let msgs_tracker = use_context::<RwSignal<Option<Id>>>().unwrap();
    let msg_event = use_context::<RwSignal<MsgEvent>>().unwrap();

    // -- Action to create test room on click
    create_effect(move |_| {
        trace!("create_effect for `New Menu`");
        let new = new_list_signal.get();
        match new {
            NewList::None => { trace!("Clicked NewList::None"); },
            NewList::Room => {
                trace!("Clicked NewList::Room");
                let room_view = RoomViewData::new_from_click();

                APP.with(|app| {
                    batch(|| {
                        app.accounts.update(|accs| { accs.insert(room_view.owner.acc_id.id, room_view.owner.clone());} );
                        app.rooms.update(|rooms| { rooms.insert(room_view.idx(), RwSignal::new(room_view.clone()));} );
                        // app.rooms_msgs.update(|chunks| {
                        //     let mut def_chunks = RoomMsgChunks::default();
                        //     def_chunks.room_id = room_view.room_id.clone();
                        //     chunks.insert(room_view.room_id.id.clone(), RwSignal::new(def_chunks));
                        // });
                        app.rooms_tabs.update(|tabs| {
                            tabs.insert(room_view.room_id.id, (room_view.idx(), room_view.view_id));
                        });
                    });
                    trace!("Created and inserted test RoomCtx")
                });

                
            },
            NewList::Msg => {
                trace!("Clicked NewList::Msg");
                let state = use_context::<Rc<ChatState>>().unwrap();
                // -- Get active room
                if let Some(active) = state.active_room.get_untracked() {
                    // -- Get some account from that room
                    let acc = state.rooms.with_untracked(|r| {
                        let room = r.get(&active.id).unwrap();
                        if room.members.is_empty() {
                            room.owner.clone()
                        } else {
                            room.members.values().next().unwrap().clone()
                        }
                    });
                    // -- Create Msgs
                    let mut msgs_vec = Vec::with_capacity(40);
                    for _ in 0..40 {
                        sleep(Duration::from_millis(2));
                        let msg = MsgCtx::new_from_click(&active, &acc);
                        msgs_vec.push(msg);
                    }
                    trace!("Created New MsgCtx");
                    // -- Save it on data
                    let last = msgs_vec.last().unwrap().room.clone();
                    // -- Save it as chunks
                    state.rooms_msgs.with_untracked(|rooms| {
                        if let Some(room) = rooms.get(&active.id) {
                            batch(|| {
                                for each in msgs_vec {
                                    room.update(|chunks| chunks.append_new_msg(each))
                                }
                            });
                        trace!("Inserted msgs to state.room_msgs {}", active);
                        // println!("{active} stats:\ntotal_msgs: {}", room.get_untracked().total_msgs);
                        } else {
                            error!("Unable to insert msgs to state.room_msgs {}", active)
                        }
                    });
                    // -- Notify all subscribers that this room got an update
                    // msgs_view.set(MsgView::NewMsg(last));
                }
            },
            NewList::Account => {
                trace!("Clicked NewList::Account");
                let state = use_context::<Rc<ChatState>>().unwrap();
                if let Some(acc) = Account::new_from_click() {
                    state.accounts.update(|accs| { accs.insert(acc.acc_id.id.clone(), acc); })
                }
            }
        }
    });
    
    let new_menu = "New".button().popout_menu(move || {
        Menu::new("")
            .entry(MenuItem::new("Msg").action(move || {
                new_list_signal.set(NewList::Msg);
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
            new_menu,
            edit_menu,
            "Settings".button().action(move || {}),
            "About".button().action(move || {})
        )).style(|s| s
            .justify_content(AlignContent::Start)
            .padding(5.)
            .row_gap(5.)
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