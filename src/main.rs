#![allow(unused)]
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::rc::Rc;

use config::launch_with_config;
use cont::acc::Account;
use floem::menu::{Menu, MenuItem};
use floem::prelude::*;
use floem::reactive::{create_effect, provide_context, use_context, Trigger};
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
    /// List of all user rooms.
    pub rooms: RwSignal<BTreeMap<Ulid, RoomCtx>>,
    /// An active room (if any).
    pub active: RwSignal<Option<(Id, Account)>>,
    /// Map with:
    /// K: [Id] of a room
    /// V: Ordered map of [MsgCtx] with its Id.
    pub data: RwSignal<HashMap<Ulid, RefCell<BTreeMap<Ulid, MsgCtx>>>>,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            rooms: RwSignal::new(BTreeMap::new()),
            active: RwSignal::new(None),
            data: RwSignal::new(HashMap::new())
        }
    }
}

// MARK: MAIN

// -----------------------
fn main() {
    provide_context(Rc::new(ChatState::new()));
    provide_context(Trigger::new());
    launch_with_config(app_view)
}

fn app_view() -> impl IntoView {
    (toolbar_view(), central_view())
        .v_stack().style(|s| s.size_full().max_size_full())
        // .container()
        // .style(|s| s.size_full().max_width_full().max_height_full())
}
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
    // Account
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
    let msgs_tracker = use_context::<Trigger>().unwrap();

    // -- Action to create test room on click
    create_effect(move |_| {
        trace!("create_effect for `New Menu`");
        let new = new_list_signal.get();
        match new {
            NewList::None => { trace!("Clicked NewList::None"); },
            NewList::Room => {
                trace!("Clicked NewList::Room");
                let state = use_context::<Rc<ChatState>>().unwrap();
                let room = RoomCtx::new_from_click();
                state.rooms.update(|s| {s.insert(room.id.id.clone(), room.clone());} );
                state.data.update(|d| { d.insert(room.id.id.clone(), RefCell::new(BTreeMap::new())); });
                trace!("Created and inserted test RoomCtx")
            },
            NewList::Msg => {
                trace!("Clicked NewList::Msg");
                let state = use_context::<Rc<ChatState>>().unwrap();
                if let Some(active) = state.active.get_untracked() {
                    let msg = MsgCtx::new_from_click(&active.0, &active.1);
                    trace!("Created New MsgCtx");
                    state.data.update(|rooms| {
                        if rooms
                        .get_mut(&active.0.id)
                        .unwrap()
                        .get_mut()
                        .insert(msg.msg.msg_id.id, msg.clone())
                        .is_none() {
                            trace!("Inserted MsgCtx to state.rooms {}", active.0)
                        }
                    });
                    msgs_tracker.notify()
                }
            },
        }
    });

    // -- Action to create test room on click
    // create_effect(move |_| {
    //     trace!("create_effect for `Edit Menu`");
    //     debug!("Clicked EditMenu::{:?}", edit_list_signal.get());
    // });
    
    let new_menu = "New".button().popout_menu(move || {
        Menu::new("")
            .entry(MenuItem::new("Room").action(move || {
                new_list_signal.set(NewList::Room);
            }))
            .separator()
            .entry(MenuItem::new("Msg").action(move || {
                new_list_signal.set(NewList::Msg);
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
    ).h_stack().style(|s| s
        .padding(5.)
        .height(30.)
        .width_full()
        .gap(5.)
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
                    .is_some_and(|a|a.0.id == r_id.id),
                    |s| s.background(Color::GRAY)
            )
        )
    }).style(|s| s
        .flex_col()
        // .min_size(0, 0)
        .width(200.)
        .min_height_full()
        .max_height_full()
        .padding(2.)
        .gap(2.)
        .border(1.)
        .border_radius(4.)
        .border_color(Color::BLACK)
    ).container()
    .style(|s| s.min_size_full().max_height_full())
    .scroll()
    // .style(|s| s.padding(10).padding_right(10))
    .scroll_style(|s| s.handle_thickness(6.))
    .debug_name("rooms_list")
}

fn central_view() -> impl IntoView {
    (rooms_view(), msg_and_editor_view())
        .h_stack()
        .style(|s| s.min_size_full().max_size_full())
}

fn msg_and_editor_view() -> impl IntoView {
    (msgs_view(), editor_view())
        .v_stack()
        .style(|s| s.min_size_full().max_size_full())
}


// MARK: msgs

fn msgs_view() -> impl IntoView {
    let state = use_context::<Rc<ChatState>>().unwrap();
    let msgs_trigger = use_context::<Trigger>().unwrap();

    let room_msgs = move || {
        trace!("->> derived_signal: msgs_view");
        msgs_trigger.track();
        if let Some(room) = state.active.get() {
            trace!("1");
            state.data.with_untracked(|rooms| {
                trace!("2");
                if let Some(msgs) = rooms.get(&room.0.id) {
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
            // msgs_trigger.track();
            let msgs = room_msgs();
            // trace!(
            //     "->> dyn_stack: room_msgs.get(): {:#?}",
            //     msgs.borrow().iter().map(|e|e.0.clone()).collect::<Vec<Ulid>>()
            // );
            msgs.into_inner()
        },
        |(id, msg)| id.clone(),
        |(id, msg)| {
            debug!("->> dyn_stack: msg");
            msg
        }
    ).style(|s| s.flex_col().padding(5.))
    .scroll()
    .style(|s| s
        .width_pct(95.)
        // .max_width_full()
        .min_height_pct(80.)
        .max_height_pct(80.)
        // .background(Color::OLIVE_DRAB)
        .border(1.)
        .border_color(Color::OLIVE)
        .border_radius(4.)
        .padding(5.)
    )
}

// MARK: editor

fn editor_view() -> impl IntoView {
    empty()
        .style(|s| s
            .width_pct(95.)
            // .max_width_full()
            .min_height_pct(20.)
            .max_height_pct(20.)
            // .background(Color::LIGHT_SALMON)
            .border(1.)
            .border_color(Color::DARK_MAGENTA)
            .border_radius(4.)
            .padding(5.)
        )
}