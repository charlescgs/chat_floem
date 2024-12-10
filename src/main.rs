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
use editor::core::selection::Selection;
use editor::text::{default_light_theme, SimpleStyling};
use floem::kurbo::Rect;
use floem::menu::{Menu, MenuItem};
use floem::prelude::*;
use floem::reactive::{batch, create_effect, provide_context, use_context, SignalRead, Trigger};
use floem::taffy::prelude::{minmax, TaffyGridLine};
use floem::taffy::{AlignContent, AlignItems, FlexDirection, GridPlacement, LengthPercentage, Line, MaxTrackSizingFunction, MinTrackSizingFunction, TrackSizingFunction};
use im::vector;
use tracing_lite::{debug, error, info, trace, warn, Level, Subscriber};
use ulid::Ulid;
use util::{Id, Tb};
use view_data::msg::MsgViewData;
use view_data::MsgEvent;
use views::chunks::RoomMsgChunks;
use views::msg::MsgCtx;
use views::msgs::msgs_view_v2;
// use views::msgs_view::main_msg_view;
use views::room::{RoomCtx, ROOM_IDX};
use views::rooms::rooms_view_v2;
use views::toolbar::toolbar_view_v2;

pub mod common;
pub mod view_data;
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
    pub mod msgs;
    pub mod msgs_view;
    pub mod room;
    pub mod rooms;
    pub mod chunks;
    pub mod toolbar;
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
    pub rooms_msgs: RwSignal<BTreeMap<Ulid, RwSignal<RoomMsgChunks>>>,
    /// 0 - no active tab (should be with `None` only).
    /// Index `0` will be checked in view_fn and special fn will be appplied.
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
        let idx = *self.rooms_tabs.read_untracked().borrow().get(&room_id).unwrap_or(&0);
        // trace!("fn: get_room_idx for {room_id} returned {idx}");
        idx
    }
}

// MARK: MAIN

// -----------------------
fn main() {
    Subscriber::new_with_max_level(Level::TRACE).with_short_time_format();
    provide_context(Rc::new(ChatState::new())); // Main UI state
    provide_context(RwSignal::new(None::<Id>)); // Msg tracker
    provide_context(RwSignal::new(MsgView::None)); // Msg load tracker
    provide_context(RwSignal::new(MsgEvent::None)); // Msg load tracker
    launch_with_config(app_view_grid)
}


fn app_view_grid() -> impl IntoView {
    let send_msg = Trigger::new();
    let new_msg_scroll_end = Trigger::new();
    provide_context(new_msg_scroll_end);
    stack((
        toolbar_view_v2(),
        rooms_view_v2(),
        msgs_view_v2(),
        // tab_msgs_view(new_msg_scroll_end),
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


// MARK: rooms

fn rooms_view() -> impl IntoView {
    info!("->> rooms_view");
    let state = use_context::<Rc<ChatState>>().unwrap();
    let active_room = state.active_room;
    let rooms = state.rooms;
    let _rooms_tabs = state.rooms_tabs;
    stack((
        dyn_stack(
            move || rooms.get(),
            move |(room_id, _)| *room_id,
            move |(room_id, room)| {
                room.style(move |s| s.apply_if(
                    active_room.get().is_some_and(|act|act.id == room_id), |s| s
                        .background(Color::LIGHT_GRAY)
                        .border(2)
                        .border_color(Color::DARK_BLUE)
                ))

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


#[derive(Debug, Clone)]
pub enum MsgView {
    None,
    /// With RoomId.
    NewMsg(Id),
    LoadMore(Rect),
    // MsgUpdated((Id, Id)) // 0: RoomId, 1: MsgId
}

// MARK: tab_msgs

fn tab_msgs_view(_new_msg_scroll_end: Trigger) -> impl IntoView {
    let state = use_context::<Rc<ChatState>>().unwrap();
    let state2 = state.clone();
    let state3 = state.clone();
    let msg_view = use_context::<RwSignal<MsgView>>().unwrap();
    let _scroll_pos = RwSignal::new(Rect::default());
    
    let act_room = state.active_room.clone();
    let rooms = state.rooms_msgs.clone();

    stack((
        tab(move || {
            match act_room.get() {
                Some(id) => {
                    trace!("tab: active_fn: Some({id})");
                    state2.get_room_idx(&id.id)
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
            move |(x, _)| {
                let idx = state3.get_room_idx(x);
                trace!("tab: key_fn: {idx}");
                idx
            }
            ,
            move |(k, v)| {
                info!("|| tab: view_fn for room:{k}");
                // main_msg_view(msg_view, v)
                let room_msgs = move || {
                    debug!("Computing room_msgs..");
                    match msg_view.get() {
                        MsgView::None => vector!(),
                        MsgView::NewMsg(room) => {
                            trace!("0. NewMsg({room})");
                            let mut chunk_iter = vector!();
                            for each in v.get_untracked().reload() {
                                trace!("1. Loading each");
                                chunk_iter.append(each.msgs.clone());
                            }
                            trace!("room_msgs: reloaded msgs({})", chunk_iter.len());
                            // for m in &chunk_iter {
                            //     println!("{} - {}", m.msg.created.human_formatted(), m.msg.text.current);
                            // }
                            // new_msg_scroll_end.notify();
                            chunk_iter
                        },
                        MsgView::LoadMore(_rect) => {
                            trace!("0. LoadMore()");
                            let mut chunk_iter = vector!();
                            for each in v.get_untracked().load_next() {
                                trace!("1. Loading each(more)");
                                chunk_iter.append(each.msgs.clone());
                            }
                            // for m in &chunk_iter {
                            //     println!("{} - {}", m.msg.created.human_formatted(), m.msg.text.current);
                            // }
                            trace!("room_msgs: loaded more msgs({})", chunk_iter.len());
                            chunk_iter
                        }
                    }
                };
            
                dyn_stack(
                    move || {
                        let chunks = room_msgs();
                        info!("->> msg fn ({} msgs)", chunks.len());
                        chunks.into_iter().rev().enumerate()
                    },
                    |(idx, _)| *idx,
                    |(_, msg)| {
                        trace!("dyn_stack: msg (view_fn): {}", msg.id);
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
            }
        ).debug_name("msgs view")
        .style(|s| s.size_full())
        .on_resize(move |_rect| {
            // scroll_pos.set(rect);
        }),
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

// MARK: text_editor

fn text_editor_view(send_msg: Trigger, _new_msg_scroll_end: Trigger) -> impl IntoView {
    let state = use_context::<Rc<ChatState>>().unwrap();
    // let msgs_tracker = use_context::<RwSignal<Option<Id>>>().unwrap();
    let msg_view = use_context::<RwSignal<MsgView>>().unwrap();
    let _editor_focus = RwSignal::new(false);

    let editor = text_editor("")
        .placeholder("Type message..")
        .styling(SimpleStyling::new())
        .style(|s| s.size_full())
        .editor_style(default_light_theme)
        .editor_style(|s| s.hide_gutter(true));
    let doc_signal = editor.editor().doc_signal();
    // let editor_focus_view = editor.editor().editor_view_id;
   
    create_effect(move |_| {
        info!("effect: create msg");
        send_msg.track();
        let text = doc_signal.with_untracked(|doc| {
            doc.rope_text().text.to_string()
        });
        if text.is_empty() {
            warn!("Text is empty");
            return
        };
        // -- Get active room
        if let Some(active_room) = state.active_room.get_untracked() {
            info!("    ..for {active_room}");
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
            let new_msg = MsgViewData::new(new_msg, &msg_author, owner);
            // trace!("new_msg_text and date: {} {}", new_msg.msg.text.current, new_msg.msg.created.human_formatted());
            // state.data.with_untracked(|rooms| {
            //     if rooms
            //         .get(&active_room.id)
            //         .unwrap()
            //         .borrow_mut()
            //         .insert(new_msg.msg.msg_id.id, new_msg.clone())
            //         .is_none() {
            //             // trace!("Inserted new MsgCtx to state.data")
            //         } else {
            //             error!("failed to insert new MsgCtx to state.data")
            //         }
            // });
            // -- Save it as chunk
            let is_success = state.rooms_msgs.with_untracked(|rooms| {
                if let Some(room) = rooms.get(&active_room.id) {
                    room.update(|r| r.append_new_msg(new_msg.clone()));
                    trace!("Inserted new MsgCtx to state.rooms_msgs");
                    return true
                }
                false
            });
            if !is_success {
                state.rooms_msgs.update(|rooms| {
                    if rooms.insert(active_room.id, RwSignal::new(RoomMsgChunks::new_from_single_msg(new_msg))).is_none() {
                        trace!("Inserted new MsgCtx to state.rooms_msgs and created RoomMsgChunks")
                    }    
                })
            }
            
            // msgs_tracker.set(Some(active_room));
            msg_view.set(MsgView::NewMsg(active_room));
            
            // new_msg_scroll_end.notify();
            
            doc_signal.with_untracked(|doc| {
                let text_len = doc.text().len();
                // trace!("text len: {text_len}");
                doc.edit_single(Selection::region(0, text_len), "", EditType::DeleteSelection);
            });
            
            // editor_focus_view.with_untracked(|efv| {
            //     if let Some(view_id) = efv {
            //         info!("editor focus requested");
            //         view_id.request_focus()
            //     }
            // })
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
            })
            // .clear_focus(move || send_msg.track()),
            , button("Attach")
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