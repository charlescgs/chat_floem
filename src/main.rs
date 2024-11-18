#![allow(unused)]
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::rc::Rc;

use config::launch_with_config;
use cont::acc::Account;
use editor::text::{default_light_theme, SimpleStyling};
use floem::menu::{Menu, MenuItem};
use floem::prelude::*;
use floem::reactive::{create_effect, provide_context, use_context};
use floem::taffy::{AlignItems, FlexDirection};
use tracing_lite::{debug, trace};
use ulid::Ulid;
use util::Id;
use views::msg::MsgCtx;
use views::room::RoomCtx;

pub mod element;
pub mod config;
pub mod cont {
    pub mod msg;
    pub mod room;
    pub mod acc;
}
pub mod util;
pub mod views {
    pub mod msg;
    pub mod room;
}

pub const SIDEBAR_WIDTH: f64 = 150.0;
pub const TOPBAR_HEIGHT: f64 = 35.0;
pub const BG: Color = Color::rgb8(180, 180, 180);
pub const BUTTON_HOVER: Color = Color::rgb8(250, 250, 0);
pub const BUTTON_ACTIVE: Color = Color::rgb8(250, 0, 0);



#[derive(Debug)]
pub struct ChatState {
    /// List of all users.
    pub accounts: RwSignal<HashMap<Ulid, Account>>,
    /// List of all user rooms.
    pub rooms: RwSignal<BTreeMap<Ulid, RoomCtx>>,
    /// An active room (if any).
    pub active: RwSignal<Option<Id>>,
    /// Map with:
    /// K: [Ulid] of a room
    /// V: Ordered map of [MsgCtx] with its Id.
    pub data: RwSignal<HashMap<Ulid, RefCell<BTreeMap<Ulid, MsgCtx>>>>,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            accounts: RwSignal::new(HashMap::new()),
            rooms: RwSignal::new(BTreeMap::new()),
            active: RwSignal::new(None),
            data: RwSignal::new(HashMap::new()),
        }
    }
}

// MARK: MAIN

// -----------------------
fn main() {
    // -- Main UI state
    provide_context(Rc::new(ChatState::new()));
    // -- Msg tracker
    provide_context(RwSignal::new(None::<Id>));
    // provide_context(Trigger::new());
    launch_with_config(app_view)
}

fn app_view() -> impl IntoView {
    stack((left_view(), right_view()))
        .debug_name("main")
        .style(|s| s
            .size_full()
            .padding(5.)
            .gap(5.)
        )
}
// fn app_view() -> impl IntoView {
//     (left_view(), right_view())
//         .h_stack()
//         .style(|s| s
//             .size_full()
//             .min_size_full()
//             .max_size_full()
//         )
// }
// -----------------------

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

impl Display for EditList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditList::None => f.write_str("None"),
            EditList::Room => f.write_str("Room"),
            EditList::Msg => f.write_str("Msg"),
            EditList::Account => f.write_str("Account"),
        }
    }
}

// MARK: toolbar

fn toolbar_view() -> impl IntoView {
    let edit_list_signal = RwSignal::new(EditList::None);
    let new_list_signal = RwSignal::new(NewList::None);
    // let msgs_tracker = use_context::<Trigger>().unwrap();
    // -- Id is a room, that got an update
    let msgs_trackerv2 = use_context::<RwSignal<Option<Id>>>().unwrap();

    // -- Action to create test room on click
    create_effect(move |_| {
        trace!("create_effect for `New Menu`");
        let new = new_list_signal.get();
        match new {
            NewList::None => { trace!("Clicked NewList::None"); },
            NewList::Room => {
                trace!("Clicked NewList::Room");
                let state = use_context::<Rc<ChatState>>().unwrap();

                let room = RoomCtx::new_from_click(state.clone());
                state.accounts.update(|accs| { accs.insert(room.owner.acc_id.id.clone(), room.owner.clone());} );
                state.rooms.update(|rooms| { rooms.insert(room.id.id.clone(), room.clone());} );
                state.data.update(|data| { data.insert(room.id.id.clone(), RefCell::new(BTreeMap::new())); });
                trace!("Created and inserted test RoomCtx")
                
            },
            NewList::Msg => {
                trace!("Clicked NewList::Msg");
                let state = use_context::<Rc<ChatState>>().unwrap();
                // -- Get active room
                if let Some(active) = state.active.get_untracked() {
                    // -- Get some account from that room
                    let acc = state.rooms.with_untracked(|r| {
                        let room = r.get(&active.id).unwrap();
                        if room.members.is_empty() {
                            room.owner.clone()
                        } else {
                            room.members.values().next().unwrap().clone()
                        }
                    });
                    // -- Create Msg
                    let msg = MsgCtx::new_from_click(&active, &acc);
                    trace!("Created New MsgCtx");
                    // -- Save it
                    state.data.with_untracked(|rooms| {
                        if rooms
                        .get(&active.id)
                        .unwrap()
                        .borrow_mut()
                        .insert(msg.msg.msg_id.id, msg.clone())
                        .is_none() {
                            trace!("Inserted MsgCtx to state.rooms {}", active)
                        }
                    });
                    // -- Notify all subscribers that this room got an update
                    msgs_trackerv2.set(Some(msg.room));
                }
            },
            NewList::Account => {
                trace!("Clicked NewList::Account");
                let state = use_context::<Rc<ChatState>>().unwrap();
                if let Some(acc) = Account::new_from_click() {
                    state.accounts.update(|accs| { accs.insert(acc.acc_id.id.clone(), acc); });

                }
            }
        }
    });

    // -- Action to create test room on click
    // create_effect(move |_| {
    //     trace!("create_effect for `Edit Menu`");
    //     debug!("Clicked EditMenu::{:?}", edit_list_signal.get());
    // });
    
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
    (
        new_menu,
        edit_menu,
        "Settings".button().action(move || {}),
        "About".button().action(move || {})
    ).h_stack()
    .debug_name("toolbar")
    .style(|s| s
        // .justify_center()
        .justify_between()
        // .padding(5.)
        // .height(35.)
        // .width_full()
        // .gap(5.)
    )
}

// MARK: rooms

fn rooms_view() -> impl IntoView {
    let state = use_context::<Rc<ChatState>>().unwrap();
    // let active = move || state.active.get();
    let state2 = state.clone();
    dyn_stack(move || state.rooms.get(), |(s, _)| s.clone(), move |(s, r)| {
        let state3 = state2.clone();
        let r_id = r.id.clone();
        r.style(move |s| 
            s.apply_if(
                // state3.active.track();
                state3.active
                    .get()
                    .is_some_and(|a|a.id == r_id.id),
                    |s| s.background(Color::LIGHT_GRAY).border(2).border_color(Color::DARK_BLUE)
            )
        )
    }).debug_name("rooms list")
    .style(|s| s
        .flex_col()
        .width_full()
        .column_gap(5.)
    )
    // .style(|s| s.min_size_full().max_height_full())
    .scroll()
    .debug_name("rooms scroll")
    .style(|s| s
        .size_full()
        .padding(5.)
        .padding_right(7.)
    )
    .scroll_style(|s| s.handle_thickness(6.).shrink_to_fit())
}

fn left_view() -> impl IntoView {
    (
        toolbar_view()
            .debug_name("toolbar")
            .style(|s| s
                .align_items(AlignItems::Center)
                .justify_between()
                .border_bottom(0.5)
                .border_color(Color::BLACK)
                .padding(5.)
                .size_full()
                .flex_basis(35.)
                .flex_grow(0.)
                .flex_shrink(0.)
            ),
        rooms_view()
    )
        .v_stack()
        .debug_name("left")
        .style(|s| s
            .border(2.)
            .border_color(Color::BLACK)
            .border_radius(5.)
            .flex_grow(0.)
            .flex_shrink(0.)
            .flex_basis(200.)
        ).clip()
}

fn right_view() -> impl IntoView {
    (msgs_view(), editor_view())
        .v_stack()
        .debug_name("right")
        .style(|s| s
            .flex_direction(FlexDirection::Column)
            .border(2.)
            .border_color(Color::NAVY)
            .border_radius(5.)
            .flex_grow(1.)
            .flex_shrink(1.)
            .flex_basis(500.)
        ).clip().style(|s| s.size_full())
}

fn msg_and_editor_view() -> impl IntoView {
    // let msgs = (msgs_view(),)
    //     .h_stack()
    //     .debug_name("msgs")
    //     .style(|s| s
    //         .size_full()
    //         .min_size_full()
    //         .max_size_full()
    //         // .flex_basis(1)
    //     );
    (editor_view(), msgs_view())
        .v_stack()
        .debug_name("msgs and editor")
        .style(|s| s
            .size_full()
            .flex_direction(FlexDirection::ColumnReverse)
            // .align_content(AlignContent::Stretch)
            // .flex_basis(-2)
            // .min_size_full()
            .max_size_full()
            // .flex_basis(0)
        )
}


// MARK: msgs

fn msgs_view() -> impl IntoView {
    let state = use_context::<Rc<ChatState>>().unwrap();
    // let msgs_trigger = use_context::<Trigger>().unwrap();
    let msgs_triggerv2 = use_context::<RwSignal<Option<Id>>>().unwrap();

    let room_msgs = move || {
        trace!("->> derived_signal: msgs_view");
        if let Some(room) = msgs_triggerv2.get() {
            trace!("1");
            state.data.with_untracked(|rooms| {
                trace!("2");
                if let Some(msgs) = rooms.get(&room.id) {
                    trace!("3");
                    // debug!("{:#?}", room_cx);
                    msgs.clone()
                } else {
                    trace!("4");
                    RefCell::new(BTreeMap::<Ulid, MsgCtx>::new())
                }
            })
        } else {
            trace!("5");
            RefCell::new(BTreeMap::<Ulid, MsgCtx>::new())
        }
    };

    dyn_stack(
        move || {
            let msgs = room_msgs();
            msgs.into_inner().into_iter().rev()
        },
        |(id, _msg)| id.clone(),
        |(_id, msg)| {
            debug!("->> dyn_stack: msg");
            let id = msg.id.id.0;
            // let _is_owner = msg.room_owner;
            msg
                .style(move |s| s.apply_if(
                    id % 2 == 0, // for now
                    // is_owner,
                    |s| s.align_self(AlignItems::End)
                ))
            }
    ).debug_name("msgs list")
    .style(|s| s
        .flex_direction(FlexDirection::ColumnReverse)
        .width_full()
        .min_height_full()
        .align_items(AlignItems::Start)
        .column_gap(5.)
    )
    .scroll()
    .debug_name("msgs scroll")
    .scroll_to_percent(move || {
        msgs_triggerv2.track();
        100.
    })
    .style(|s| s
        .size_full()
        .padding(5.)
        .padding_right(7.)
    )
    .scroll_style(|s| s.handle_thickness(6.).shrink_to_fit())
}

// MARK: editor

fn editor_view() -> impl IntoView {
    let editor = text_editor("New message")
    .styling(SimpleStyling::new())
    .style(|s| s.size_full())
    .editor_style(default_light_theme)
    .editor_style(|s| s.hide_gutter(true));

    h_stack((
        container(editor).style(|s| s
            .flex_grow(1.)
            .flex_shrink(2.)
            .flex_basis(300.)
            .min_width(100.)
            .border(0.5)
            .border_color(Color::NAVY)
            .border_radius(5.)
            .padding(5.)
        ),
        v_stack((
            button("Send"),
            button("Attach"),
        )).debug_name("editor buttons")
        .style(|s| s
            .flex_grow(0.)
            .flex_shrink(0.)
            .flex_basis(50.)
            .border(0.5)
            .border_color(Color::NAVY)
            .border_radius(5.)
            .padding(2.)
            .column_gap(2.)
        ),
    )).debug_name("text editor")
    .style(|s| s
        .flex_basis(100.)
        .flex_grow(0.)
        .flex_shrink(0.)
        .border(1.)
        .border_color(Color::DARK_GREEN)
        .border_radius(5.)
    )
}