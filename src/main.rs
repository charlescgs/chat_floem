#![allow(unused)]
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use chrono_lite::Datetime;
use config::launch_with_config;
use cont::msg::{Msg, Text};
use editor::core::editor::EditType;
use editor::core::selection::Selection;
use editor::text::{default_light_theme, SimpleStyling};
use floem::prelude::*;
use floem::reactive::{create_effect, provide_context, Trigger};
use floem::taffy::prelude::{minmax, TaffyGridLine};
use floem::taffy::{GridPlacement, LengthPercentage, Line, MaxTrackSizingFunction, MinTrackSizingFunction, TrackSizingFunction};
use tracing_lite::{info, trace, warn, Level, Subscriber};
use util::{Id, Tb};
use view_data::msg::MsgViewData;
use view_data::session::APP;
use view_data::MsgEvent;
use views::msgs::msgs_view_v2;
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
    // pub mod room;
    pub mod rooms;
    pub mod chunks;
    pub mod toolbar;
}

pub const SIDEBAR_WIDTH: f64 = 150.0;
pub const TOPBAR_HEIGHT: f64 = 35.0;
pub const BG: Color = Color::rgb8(180, 180, 180);
pub const BUTTON_HOVER: Color = Color::rgb8(250, 250, 0);
pub const BUTTON_ACTIVE: Color = Color::rgb8(250, 0, 0);


// MARK: MAIN

// -----------------------
fn main() {
    Subscriber::new_with_max_level(Level::TRACE).with_short_time_format();
    provide_context(RwSignal::new(None::<Id>)); // Msg tracker
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

// MARK: text_editor

fn text_editor_view(send_msg: Trigger, _new_msg_scroll_end: Trigger) -> impl IntoView {
    let active_room = APP.with(|app| app.active_room);
    let rooms = APP.with(|app| app.rooms);
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
        if let Some(active_room) = active_room.get_untracked() {
            info!("    ..for {}", active_room.id);
            // -- Get message author (dummy for now)
            let (msg_author, owner) = {
                let room = rooms.get_untracked();
                let room = room.get(&active_room.idx).unwrap();
                if room.members.is_empty() {
                    (room.owner.clone(), true)
                } else {
                    (room.members.values().last().unwrap().clone(), false)
                }
            };
    
            // -- Create new message
            let new_msg = Msg {
                msg_id: Id::new(Tb::Msg),
                room_id: active_room.id(),
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
            let is_success = rooms.with_untracked(|rooms| {
                if let Some(room) = rooms.get(&active_room.idx) {
                    room.msgs.update(|r| r.append_new_msg(new_msg.clone()));
                    trace!("Inserted new MsgCtx to state.rooms");
                    return true
                }
                false
            });
            // if !is_success {
            //     rooms.update(|rooms| {
            //         if rooms.insert(active_room.idx, RoomMsgChunks::new_from_single_msg(new_msg)).is_none() {
            //             trace!("Inserted new MsgCtx to state.rooms_msgs and created RoomMsgChunks")
            //         }    
            //     })
            // }
            
            // msgs_tracker.set(Some(active_room));
            // msg_view.set(MsgView::NewMsg(active_room));
            
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