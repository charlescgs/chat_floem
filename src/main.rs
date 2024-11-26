#![allow(unused)]
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::thread::sleep;
use std::time::Duration;

use chrono_lite::Datetime;
use config::launch_with_config;
use cont::acc::Account;
use cont::msg::{Msg, Text};
use editor::core::editor::EditType;
use editor::core::selection::{SelRegion, Selection};
use editor::text::{default_light_theme, SimpleStyling};
use floem::kurbo::Rect;
use floem::menu::{Menu, MenuItem};
use floem::prelude::*;
use floem::reactive::{batch, create_effect, provide_context, use_context, Trigger};
use floem::taffy::prelude::{minmax, TaffyGridLine};
use floem::taffy::{AlignContent, AlignItems, FlexDirection, GridPlacement, LengthPercentage, Line, MaxTrackSizingFunction, MinTrackSizingFunction, TrackSizingFunction};
use im_rc::vector;
use tracing_lite::{debug, info, trace, Level, Subscriber};
use ulid::Ulid;
use util::{Id, Tb};
use views::msg::{MsgCtx, RoomMsgChunks};
use views::msg_view::main_msg_view;
use views::room::{RoomCtx, ROOM_IDX};

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
    pub mod msg_view;
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
    pub active_room: RwSignal<Option<Id>>,
    // / Stores info what range of its msgs is loaded.
    // pub active_room_msgs_data: RwSignal<RoomMsgChunks>,
    /// Map with:
    /// K: [Ulid] of a room
    /// V: Ordered map of [MsgCtx] with its Id.
    pub data: RwSignal<HashMap<Ulid, RefCell<BTreeMap<Ulid, MsgCtx>>>>,
    /// List of all tabs with chunked msgs.
    pub rooms_msgs: RwSignal<BTreeMap<Ulid, Rc<RefCell<RoomMsgChunks>>>>,
    
    // An active room (if any).
    // pub tab_active_room: RwSignal<Option<(Id, usize)>>,
    /// 0 - no active tab (should be with `None` only).
    pub rooms_tabs: RwSignal<HashMap<Ulid, usize>>
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            accounts: RwSignal::new(HashMap::new()),
            rooms: RwSignal::new(BTreeMap::new()),
            active_room: RwSignal::new(None),
            data: RwSignal::new(HashMap::new()),
            // active_room_msgs_data: RwSignal::new(RoomMsgChunks::default()),
            rooms_msgs: RwSignal::new(BTreeMap::new()),
            rooms_tabs: RwSignal::new(HashMap::new())
        }
    }

    /// Obtain room index for the tab to use.
    /// 
    /// Index 0 equals `No active Tabs`.
    pub fn get_room_idx(&self, room_id: &Ulid) -> usize {
        let idx = *self.rooms_tabs.get_untracked().get(&room_id).unwrap_or(&0);
        trace!("fn: get_room_idx for {room_id} returned {idx}");
        idx
    }
}

// MARK: MAIN

// -----------------------
fn main() {
    Subscriber::new_with_max_level(Level::DEBUG).with_short_time_format();
    provide_context(Rc::new(ChatState::new())); // Main UI state
    provide_context(RwSignal::new(None::<Id>)); // Msg tracker
    provide_context(RwSignal::new(MsgView::None)); // Msg load tracker
    launch_with_config(app_view_grid)
}


fn app_view_grid() -> impl IntoView {
    let send_msg = Trigger::new();
    let new_msg_scroll_end = Trigger::new();
    provide_context(new_msg_scroll_end);
    stack((
        toolbar_view(),
        rooms_view(),
        // msgs_view(),
        tab_msgs_view(new_msg_scroll_end),
        text_editor_view(send_msg, new_msg_scroll_end),
        editor_toolbar_view(send_msg),
    ))
        .debug_name("grid container")
        .style(|s| s
            .grid()
            .grid_template_columns(vec![
                TrackSizingFunction::Single(
                    minmax(
                        MinTrackSizingFunction::Fixed(LengthPercentage::Length(200.0)),
                        MaxTrackSizingFunction::Fraction(0.)
                    ),
                ),
                TrackSizingFunction::Single(
                    minmax(
                        MinTrackSizingFunction::Auto,
                        MaxTrackSizingFunction::Auto
                    ),
                ),
                TrackSizingFunction::Single(
                    minmax(
                        MinTrackSizingFunction::Fixed(LengthPercentage::Length(60.0)),
                        MaxTrackSizingFunction::Fraction(0.)
                    ),
                )
            ])
            .grid_template_rows(Vec::from([
                TrackSizingFunction::Single(
                    minmax(
                        MinTrackSizingFunction::Fixed(LengthPercentage::Length(40.0)),
                        MaxTrackSizingFunction::Fraction(0.)
                    )
                ),
                    TrackSizingFunction::Single(
                        minmax(
                            MinTrackSizingFunction::Auto,
                            MaxTrackSizingFunction::Auto
                    )
                ),
                    TrackSizingFunction::Single(
                        minmax(
                            MinTrackSizingFunction::Auto,
                            MaxTrackSizingFunction::Auto
                    )
                ),
                    TrackSizingFunction::Single(
                        minmax(
                            MinTrackSizingFunction::Fixed(LengthPercentage::Length(80.0)),
                            MaxTrackSizingFunction::FitContent(LengthPercentage::Length(100.))
                    )
                )
            ]))
            .column_gap(5.)
            .row_gap(5.)
            .size_full()
            .padding(5.)

            .border(2.)
            .border_color(Color::BLACK)
            .border_radius(5.)
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
    let msgs_view = use_context::<RwSignal<MsgView>>().unwrap();

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
                    state.rooms_msgs.update(|chunks| {
                        let mut def_chunks = RoomMsgChunks::default();
                        def_chunks.room_id = room.id.clone();
                        chunks.insert(room.id.id.clone(), Rc::new(RefCell::new(def_chunks)));
                    });
                    state.rooms_tabs.update(|tabs| {
                        let new_idx = ROOM_IDX.fetch_add(1, Ordering::Relaxed);
                        tabs.insert(room.id.id, new_idx);
                    });
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
                        sleep(Duration::from_millis(2));
                        let msg = MsgCtx::new_from_click(&active, &acc);
                        msgs_vec.push(msg);
                    }
                    trace!("Created New MsgCtx");
                    // -- Save it on data
                    let last = msgs_vec.last().unwrap().room.clone();
                    state.data.with_untracked(|rooms| {
                        let mut map = rooms.get(&active.id)
                            .unwrap()
                            .borrow_mut();
                        for each in msgs_vec.clone() {
                            map.insert(each.id.id, each);
                        }
                        trace!("Inserted MsgCtx to state.rooms {}", active)
                    });
                    // -- Save it as chunks
                    state.rooms_msgs.update(|rooms| {
                        if let Some(room) = rooms.get_mut(&active.id) {
                            for each in msgs_vec {
                                room.borrow_mut().append_new_msg(each);
                            }
                            trace!("Inserted msgs to state.room_msgs {}", active)
                        }
                    });
                    // -- Notify all subscribers that this room got an update
                    // msgs_tracker.set(Some(last.clone()));
                    msgs_view.set(MsgView::NewMsg(last));
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

// MARK: rooms

fn rooms_view() -> impl IntoView {
    let state = use_context::<Rc<ChatState>>().unwrap();
    let state2 = state.clone();
    stack((
        dyn_stack(move || state.rooms.get(), |(s, _)| s.clone(), move |(s, r)| {
            let state3 = state2.clone();
            let r_id = r.id.clone();
            r.style(move |s| 
                s.apply_if(
                    state3.active_room.get().is_some_and(|a|a.id == r_id.id), |s| s
                        .background(Color::LIGHT_GRAY)
                        .border(2)
                        .border_color(Color::DARK_BLUE)
                )
            )
        }).debug_name("rooms list")
        .style(|s| s
            .flex_col()
            .width_full()
            // .max_width_full()
            .column_gap(5.)
        )
        .scroll()
        .debug_name("rooms scroll")
        .style(|s| s
            .size_full()
            .padding(5.)
            .padding_right(7.)
        )
        .scroll_style(|s| s.handle_thickness(6.).shrink_to_fit()),
    )).debug_name("rooms stack")
    .style(|s| s
        .background(Color::LIGHT_BLUE)
        .border_color(Color::BLACK)
        .border(1.)
        .grid_column(Line {
            start: GridPlacement::from_line_index(1),
            end: GridPlacement::Span(1)
        })
        .grid_row(Line {
            start: GridPlacement::from_line_index(2),
            end: GridPlacement::Span(3)
        })
    )
}

// MARK: msgs


#[derive(Debug, Clone)]
pub enum MsgView {
    None,
    NewMsg(Id),
    LoadMore(Rect)
}


fn msgs_view() -> impl IntoView {
    let state = use_context::<Rc<ChatState>>().unwrap();
    let state2 = state.clone();
    let state3 = state.clone();
    let msgs_tracker = use_context::<RwSignal<Option<Id>>>().unwrap();
    let msg_view = use_context::<RwSignal<MsgView>>().unwrap();
    let scroll_pos = RwSignal::new(Rect::default());
    let load_more = Trigger::new();

    let room_msgs = move || {
        debug!("->> Compute room_msgs");
        match msg_view.get() {
            MsgView::None => vector!(),
            MsgView::NewMsg(room) => {
                let room_chunks = state.rooms_msgs.with_untracked(|rooms| {
                    let chunks = if let Some(msgs) = rooms.get(&room.id) {
                       msgs.clone() 
                    } else {
                        let mut room_chunks = Rc::new(RefCell::new(RoomMsgChunks::default()));
                        room_chunks.borrow_mut().room_id = room.clone();
                        state.rooms_msgs.update(|rooms| { rooms.insert(room.id, room_chunks.clone()); });
                        room_chunks
                    };
                    chunks
                });
                let mut chunk_iter = vector!();
                for each in room_chunks.borrow().reload() {
                    chunk_iter.append(each.msgs.clone());
                }
                chunk_iter
            },
            MsgView::LoadMore(rect) => {
                let act_room = state.active_room.get_untracked();
                match act_room {
                    Some(room_id) => {
                        let room_chunks = state.rooms_msgs.with_untracked(|rooms| {
                            let chunks = if let Some(msgs) = rooms.get(&room_id.id) {
                                msgs.clone()
                            } else {
                                let mut chunks =  Rc::new(RefCell::new(RoomMsgChunks::default()));
                                chunks.borrow_mut().room_id = room_id.clone();
                                state.rooms_msgs.update(|rooms| { rooms.insert(room_id.id, chunks.clone()); });
                                chunks
                            };
                            chunks
                        });
                        let mut chunk_iter = vector!();
                        for each in room_chunks.borrow_mut().load_next() {
                            chunk_iter.append(each.msgs.clone());
                        }
                        chunk_iter
                    },
                    None => vector![]
                }
            }
        }
    };

    stack((
        dyn_stack(
            move || {
                debug!("->> msg fn");
                let chunks = room_msgs();
                chunks.into_iter().rev().enumerate()
            },
            |(idx, _)| *idx,
            |(_, msg)| {
                trace!("dyn_stack: msg (view_fn)");
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
            .flex_direction(FlexDirection::ColumnReverse)
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
        )
        .scroll_to_percent(move || {
            trace!("scroll_to_percent");
            msgs_tracker.track();
            100.0
        })
        .on_resize(move |rect| {
            scroll_pos.set(rect);
        })
        .on_scroll(move |rect| {
            if rect.y0 == 0.0 {
                debug!("on_scroll: load_more notified!");
                msg_view.set(MsgView::LoadMore(rect));
                // load_more.notify();
            }
        }),
    )).debug_name("msgs stack")
    .style(|s| s
        .padding(10.)
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

// MARK: tab_msgs

fn tab_msgs_view(new_msg_scroll_end: Trigger) -> impl IntoView {
    let state = use_context::<Rc<ChatState>>().unwrap();
    let state2 = state.clone();
    let state3 = state.clone();
    // let msgs_tracker = use_context::<RwSignal<Option<Id>>>().unwrap();
    let msg_view = use_context::<RwSignal<MsgView>>().unwrap();
    let scroll_pos = RwSignal::new(Rect::default());
    
    let act_room = state.active_room;
    let rooms = state.rooms_msgs;

    
    // create_effect(move |_| {
    //     debug!("->> effect: load_more");
        // load_more.track();
        // TODO:
        // 1. Check if there is more msgs to load.
        // 2. Load another chunk.
        // if state.active_room_msgs_data.
    // });

    stack((
        tab(
            move || {
                match act_room.get() {
                    Some(id) => state2.get_room_idx(&id.id),
                    None => 0
                }
            },
            move || rooms.get(),
            move |(x,_)| {
                let idx = state3.get_room_idx(x);
                trace!("key_fn: {idx}");
                idx
            }
            ,
            |(_, v)| {
                info!("dyn_stack: msg (view_fn) for: {}", v.borrow().room_id);
                main_msg_view(v)
                // empty()
            }
        ).debug_name("msgs view")
        .style(|s| s
            .flex_direction(FlexDirection::ColumnReverse)
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
        )
        .scroll_to_percent(move || {
            trace!("scroll_to_percent");
            new_msg_scroll_end.track();
            100.0
        })
        .on_resize(move |rect| {
            scroll_pos.set(rect);
        })
        .on_scroll(move |rect| {
            if rect.y0 == 0.0 {
                debug!("on_scroll: load_more notified!");
                msg_view.set(MsgView::LoadMore(rect));
            }
        }),
    )).debug_name("msgs stack")
    .style(|s| s
        .padding(10.)
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

// MARK: text_editor

fn text_editor_view(send_msg: Trigger, new_msg_scroll_end: Trigger) -> impl IntoView {
    let state = use_context::<Rc<ChatState>>().unwrap();
    // let msgs_tracker = use_context::<RwSignal<Option<Id>>>().unwrap();
    let msg_view = use_context::<RwSignal<MsgView>>().unwrap();
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
            info!(" for {active_room}");
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
                .insert(new_msg.msg.msg_id.id, new_msg.clone())
                .is_none() {
                    trace!("Inserted new MsgCtx to state.data")
                }
            });
            // -- Save it as chunk
            state.rooms_msgs.update(|rooms| {
                if let Some(room) = rooms.get(&active_room.id) {
                    room.borrow_mut().append_new_msg(new_msg);
                    trace!("Inserted new MsgCtx to state.rooms_msgs")
                } else {
                    if rooms.insert(active_room.id, Rc::new(RefCell::new(RoomMsgChunks::new_from_single_msg(new_msg)))).is_none() {
                        trace!("Inserted new MsgCtx to state.rooms_msgs and created RoomMsgChunks")
                    }
                }
            });
            
            // msgs_tracker.set(Some(active_room));
            msg_view.set(MsgView::NewMsg(active_room));
            new_msg_scroll_end.notify();
            
            doc_signal.update(|d| {
                let t = d.text();
                trace!("text len: {}", t.len());
                let mut sel = Selection::new();
                sel.add_region(SelRegion::new(0, t.len(), None));
                d.edit_single(sel, "", EditType::Delete);
            });
            // editor_focus.set(true);
            // editor.editor().active.set(true);
        }
    });
    
    stack((
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
    )).debug_name("text editor")
    .style(|s| s
        // .padding(5.)
        .background(Color::YELLOW)
        .border_color(Color::BLACK)
        .border(1.)
        .grid_column(Line {
            start: GridPlacement::from_line_index(2),
            end: GridPlacement::Span(1)
        })
        .grid_row(Line {
            start: GridPlacement::from_line_index(4),
            end: GridPlacement::Span(1)
        })
    )
}

// MARK: ed_toolbar

pub fn editor_toolbar_view(send_msg: Trigger) -> impl IntoView {
    stack((
        v_stack((
            button("Send").action(move || {
                send_msg.notify();
            }).clear_focus(move || send_msg.track()),
            button("Attach")
        )).style(|s| s.gap(5.)),
    )).debug_name("editor buttons")
    .style(|s| s
        .justify_center()
        .padding(5.)
        .background(Color::RED)
        .border_color(Color::BLACK)
        .border(1.)
        .grid_column(Line {
            start: GridPlacement::from_line_index(3),
            end: GridPlacement::Span(1)
        })
        .grid_row(Line {
            start: GridPlacement::from_line_index(4),
            end: GridPlacement::Span(1)
        })
    )
}