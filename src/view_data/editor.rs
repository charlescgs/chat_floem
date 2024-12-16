use std::collections::HashMap;
use std::rc::Rc;

use chrono_lite::Datetime;
use floem::{prelude::*, ViewId};
use floem::taffy::{prelude::TaffyGridLine, GridPlacement, Line};
use floem::reactive::{create_effect, Trigger};
use editor::{core::{editor::EditType, selection::Selection}};
use editor::text::{default_light_theme, Document, SimpleStyling};
use tracing_lite::{info, trace, warn};
use ulid::Ulid;

use crate::common::CommonData;
use crate::view_data::msg::MsgViewData;
use crate::util::{Id, Tb};
use crate::cont::msg::{Msg, Text};
use super::session::APP;




/// Contains all documents and state for the rooms.
pub struct EditorViewData {
    pub view_id: ViewId,
    pub open_documents: RwSignal<HashMap<Ulid, Rc<dyn Document>>>,
    // TODO: more is needed..
    pub common_data: Rc<CommonData>
}



pub fn text_editor_view(send_msg: Trigger) -> impl IntoView {
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