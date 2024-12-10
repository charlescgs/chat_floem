use std::rc::Rc;
use std::thread::sleep;
use std::time::Duration;

use chat_util::gen::{gen_u64, gen_u64_in_range};
use floem::prelude::*;
use floem::menu::{Menu, MenuItem};
use floem::reactive::{batch, SignalRead};
use floem::reactive::create_effect;
use floem::reactive::use_context;
use floem::taffy::prelude::TaffyGridLine;
use floem::taffy::{AlignContent, GridPlacement, Line};
use tracing_lite::{debug, error, info, trace};

use crate::cont::acc::Account;
use crate::view_data::msg::MsgViewData;
use crate::view_data::room::RoomViewData;
use crate::view_data::session::APP;
use crate::view_data::MsgEvent;
use crate::ChatState;


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
    let msg_event = use_context::<RwSignal<MsgEvent>>().unwrap();

    // -- Action to create test room on click
    create_effect(move |_| {
        trace!("create_effect for `New Menu`");
        let new = new_list_signal.get();
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
                                error!("value returned when attempted to insert {ret:#?}")
                            }
                        });
                        app.rooms_tabs.update(|tabs| {
                            tabs.insert(
                                room_view.room_id.id,
                                (room_view.idx(), room_view.view_id, room_view.get_update)
                            );
                        });
                    });
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
                                    info!("New msg appended!");
                                });
                                room.last_msg.set(Some(msg));
                                // -- Notify subscribers about new msg event
                                msg_event.set(MsgEvent::NewFor(room.room_id.id));
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