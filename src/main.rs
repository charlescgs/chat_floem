#![allow(unused)]
use std::cell::RefCell;
use std::collections::btree_map::Range;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::ops::{Bound, RangeInclusive};
use std::rc::Rc;

use chrono_lite::Datetime;
use config::launch_with_config;
use cont::acc::Account;
use cont::msg::{Msg, Text};
use editor::core::editor::EditType;
use editor::core::selection::{SelRegion, Selection};
use editor::text::{default_light_theme, SimpleStyling};
use editor::view::editor_view;
use floem::kurbo::Rect;
use floem::menu::{Menu, MenuItem};
use floem::prelude::*;
use floem::reactive::{batch, create_effect, create_updater, provide_context, use_context, Trigger};
use floem::style::TextOverflow;
use floem::taffy::{AlignItems, FlexDirection};
use tracing_lite::{debug, info, trace};
use ulid::Ulid;
use util::{Id, Tb};
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


/// Struct holding info regarding msgs loaded.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RangeInView {
    /// Total room msgs.
    pub total: u16,
    /// Start range.
    pub from: Ulid,
    /// End range.
    pub to: Ulid
}



#[derive(Debug)]
pub struct ChatState {
    /// List of all users.
    pub accounts: RwSignal<HashMap<Ulid, Account>>,
    /// List of all user rooms.
    pub rooms: RwSignal<BTreeMap<Ulid, RoomCtx>>,
    /// An active room (if any).
    pub active_room: RwSignal<Option<Id>>,
    /// Stores info what range of its msgs is loaded.
    pub active_room_msgs_data: RwSignal<RangeInView>,
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
            active_room: RwSignal::new(None),
            data: RwSignal::new(HashMap::new()),
            active_room_msgs_data: RwSignal::new(RangeInView::default())
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
    // -- Msg load tracker
    provide_context(RwSignal::new(None::<Id>));
    // provide_context(Trigger::new());
    launch_with_config(app_view)
}

fn app_view() -> impl IntoView {
    stack((left_view(), right_view()))
        .debug_name("main")
        .style(|s| s
            .size_full()
            .max_size_full()
            .padding(5.)
            .gap(5.)
        )
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
    // -- Id is a room, that got an update
    let msgs_tracker = use_context::<RwSignal<Option<Id>>>().unwrap();

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
                batch(|| {
                    state.accounts.update(|accs| { accs.insert(room.owner.acc_id.id.clone(), room.owner.clone());} );
                    state.rooms.update(|rooms| { rooms.insert(room.id.id.clone(), room.clone());} );
                    state.data.update(|data| { data.insert(room.id.id.clone(), RefCell::new(BTreeMap::new())); });
                });
                trace!("Created and inserted test RoomCtx")
                
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
                        let msg = MsgCtx::new_from_click(&active, &acc);
                        msgs_vec.push(msg);
                    }
                    trace!("Created New MsgCtx");
                    // -- Save it
                    let last = msgs_vec.last().unwrap().clone();
                    state.data.with_untracked(|rooms| {
                        // if rooms
                        let mut map = rooms.get(&active.id)
                            .unwrap()
                            .borrow_mut();
                        for each in msgs_vec {
                            map.insert(each.msg.msg_id.id, each);
                        }
                        // .is_none() {
                        trace!("Inserted MsgCtx to state.rooms {}", active)
                        // }
                    });
                    // -- Notify all subscribers that this room got an update
                    msgs_tracker.set(Some(last.room));
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
        .justify_between()
    )
}

// MARK: rooms

fn rooms_view() -> impl IntoView {
    let state = use_context::<Rc<ChatState>>().unwrap();
    let state2 = state.clone();
    dyn_stack(move || state.rooms.get(), |(s, _)| s.clone(), move |(s, r)| {
        let state3 = state2.clone();
        let r_id = r.id.clone();
        r.style(move |s| 
            s.apply_if(
                state3.active_room
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
        // .style(|s| s.size_full().max_size_full())
}

fn right_view() -> impl IntoView {
    (msgs_view(), chat_editor_view())
        .v_stack()
        .style(|s| s.size_full())
        .clip()
        .debug_name("right")
        .style(|s| s
            .flex_direction(FlexDirection::Column)
            .border(2.)
            .border_color(Color::NAVY)
            .border_radius(5.)
            .flex_grow(1.)
            .flex_shrink(1.)
            .flex_basis(500.)
        )
        // .clip().style(|s| s
            // .size_full().max_size_full()
        // )
}


// MARK: msgs

fn msgs_view() -> impl IntoView {
    let state = use_context::<Rc<ChatState>>().unwrap();
    let state2 = state.clone();
    let msgs_trigger = use_context::<RwSignal<Option<Id>>>().unwrap();
    let scroll_pos = RwSignal::new(Rect::default());
    let load_more = Trigger::new();

    let room_msgs = move || {
        debug!("->> derived_signal: msgs_view");
        if let Some(room) = msgs_trigger.get() {
            // trace!("1");
            state.data.with_untracked(|rooms| {
                // trace!("2");
                if let Some(msgs) = rooms.get(&room.id) {
                    // trace!("3");
                    // debug!("{:#?}", room_cx);
                    msgs.clone()
                } else {
                    // trace!("4");
                    RefCell::new(BTreeMap::<Ulid, MsgCtx>::new())
                }
            })
        } else {
            // trace!("5");
            RefCell::new(BTreeMap::<Ulid, MsgCtx>::new())
        }
    };
    
    create_effect(move |_| {
        debug!("->> effect: load_more");
        load_more.track();
        // TODO:
        // 1. Check if there is more msgs to load.
        // 2. Load another chunk.
        // if state.active_room_msgs_data.
    });

    dyn_stack(
        move || {
            let msgs = room_msgs();
            let range = state2.active_room_msgs_data.get();
            if range.total > 20 {
                let r = msgs.into_inner();
                r.range((Bound::Included(range.from), Bound::Included(range.to)))
                .map(|(k, v)| (k.clone(), v.clone())).collect() // FIXME: ugly...
            } else {
                msgs.into_inner()
            }
        },
        |(id, _msg)| id.clone(),
        |(_id, msg)| {
            trace!("dyn_stack: msg");
            let id = msg.id.id.0;
            // let _is_owner = msg.room_owner;
            msg
                .style(move |s| s
                    // .max_width_pct(80.)
                    // .text_overflow(TextOverflow::Wrap)
                    .apply_if(id % 2 == 0, // for now
                    // is_owner,
                    |s| s.align_self(AlignItems::End)
                    )
                )
            }
    ).debug_name("msgs list")
    .style(|s| s
        .flex_direction(FlexDirection::ColumnReverse)
        .width_full()
        .max_width_full()
        .min_height_full()
        .align_items(AlignItems::Start)
        .column_gap(5.)
        
    )
    .scroll()
    .debug_name("msgs scroll")
    .scroll_to_percent(move || {
        trace!("scroll_to_percent");
        msgs_trigger.track();
        100.0
    })
    .on_resize(move |rect| {
        // trace!("on_resize: {rect}");
        scroll_pos.set(rect);
    })
    .on_scroll(move |rect| {
        // trace!("on_scroll: {} + 30", rect.y0);
        if rect.y0 == 0.0 {
            debug!("on_scroll: load_more notified!");
            load_more.notify();
        }
    })
    .style(|s| s
        .max_size_full()
        .padding(5.)
        .padding_right(7.)
    )
    .scroll_style(|s| s.handle_thickness(6.).shrink_to_fit())
}

// MARK: editor

fn chat_editor_view() -> impl IntoView {
    let state = use_context::<Rc<ChatState>>().unwrap();
    let msgs_trackerv2 = use_context::<RwSignal<Option<Id>>>().unwrap();
    let send_msg = Trigger::new();
    let editor_focus = RwSignal::new(false);

    let editor = text_editor("New message")
        .styling(SimpleStyling::new())
        .style(|s| s.size_full())
        .editor_style(default_light_theme)
        .editor_style(|s| s.hide_gutter(true));
    let ed_sig = RwSignal::new(editor.editor().clone());
    let doc = editor.doc();
    let doc2 = editor.doc();
    // let ed = editor_view(ed_sig, move |e| editor_focus.get());
    
    let doc_signal = RwSignal::new(doc);

    // create_effect(move |_| {
    //     editor_focus.track();
    //     doc2.edit_single(Selection::new(), "", EditType::Other);
    // });
    
    create_effect(move |_| {
        info!("effect: create msg");
        send_msg.track();
        let text = doc_signal.with_untracked(|doc| {
            doc.rope_text().text.to_string()
        });
        if text.is_empty() { return };
        // -- Get active room
        if let Some(active_room) = state.active_room.get_untracked() {
            // -- Get message author (dummy for now)
            let (msg_author, owner) = {
                let room = state.rooms.get_untracked();
                let room = room.get(&active_room.id).unwrap();
                if room.members.is_empty() {
                    (room.owner.clone(), true)
                } else {
                    (room.members.values().last().unwrap().clone(), false)
                }
            };
    
            // -- Create new message
            let new_msg = Msg {
                msg_id: Id::new(Tb::Msg),
                room_id: active_room.clone(),
                author: msg_author.acc_id.clone(),
                created: Datetime::default(),
                sent: None,
                text: Text {
                    current: text,
                    edits: None,
                    last_edited: None
                },
                media: None,
                edited: None,
                comments: None,
                reactions: None,
                delivered_to_all: false,
                viewed_by_all: false
            };
            let new_msg = MsgCtx::new(new_msg, &msg_author, owner);
            state.data.with_untracked(|rooms| {
                if rooms
                .get(&active_room.id)
                .unwrap()
                .borrow_mut()
                .insert(new_msg.msg.msg_id.id, new_msg)
                .is_none() {
                    trace!("Inserted new MsgCtx to state.rooms")
                }
            });
            msgs_trackerv2.set(Some(active_room));
            doc_signal.update(|d| {
                let t = d.text();
                trace!("text len: {}", t.len());
                let mut sel = Selection::new();
                sel.add_region(SelRegion::new(0, t.len(), None));
                d.edit_single(sel, "", EditType::Delete);
            });
            editor_focus.set(true);
            // editor.editor().active.set(true);
        }
    });
    
    h_stack((
        container(editor)
        .style(|s| s
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
            button("Send").action(move || {
                send_msg.notify();
            }).clear_focus(move || send_msg.track()),
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