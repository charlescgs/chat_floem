use std::collections::HashMap;
use std::rc::Rc;

use chrono_lite::Datetime;
use editor::text_document::TextDocument;
use editor::Editor;
use floem::{prelude::*, AnyView, ViewId};
use floem::taffy::{prelude::TaffyGridLine, GridPlacement, Line};
use floem::reactive::{create_effect, use_context, Trigger, WriteSignal};
use editor::core::{editor::EditType, selection::Selection};
use editor::text::{default_light_theme, Document, SimpleStyling};
use tracing_lite::{error, info, trace, warn};
use ulid::Ulid;

use crate::common::CommonData;
use crate::view_data::msg::MsgViewData;
use crate::util::{Id, Tb};
use crate::cont::msg::{Msg, Text};
use super::session::APP;
use super::MsgEvent;




/// Contains all documents and state for the rooms.
pub struct EditorViewData {
    pub editor: Editor,
    pub view_id: ViewId,
    /// Map with editor documents for each room.  
    /// K: room id
    pub docs: RwSignal<HashMap<Ulid, Rc<dyn Document>>>,
    pub active_doc: RwSignal<Option<Rc<dyn Document>>>,
    // TODO: more is needed..
    pub common_data: Rc<CommonData>
}

impl EditorViewData {
    pub fn new() -> Self {
        let cx = APP.with(|app| app.provide_scope());
        let active_doc = Rc::new(TextDocument::new(cx, "default text doc"));
        Self {
            view_id: ViewId::new(),
            docs: cx.create_rw_signal(HashMap::new()),
            active_doc: cx.create_rw_signal(Some(active_doc.clone())),
            common_data: APP.with(|app| app.common_data.clone()),
            editor: Editor::new(cx, active_doc, Rc::new(SimpleStyling::new()), false)
        }
    }
}


impl IntoView for EditorViewData {
    type V = AnyView;

    fn into_view(self) -> Self::V {
        // let vid = self.view_id;
        let active_room = APP.with(|app| app.active_room);
        let rooms = APP.with(|app| app.rooms);
        let msg_event = use_context::<RwSignal<MsgEvent>>().unwrap();
        let send_msg = use_context::<Trigger>().unwrap();
        let new_room = use_context::<RwSignal<Option<Ulid>>>().unwrap();

        // let default_doc = Rc::new(TextDocument::new(Scope::current(), "def"));
        let text_editor = text_editor("")
            // .placeholder("Type message..")
            .styling(SimpleStyling::new())
            .style(|s| s.size_full())
            .editor_style(default_light_theme)
            .editor_style(|s| s.hide_gutter(true));

        let doc_signal = text_editor.editor().doc_signal();
        let ed_cx = text_editor.editor().cx.get();

        create_effect(move |_| {
            info!("->> effect: new room editor doc");
            if let Some(room_id) = new_room.get() {
                self.docs.update(|docs| {
                    docs.insert(room_id, Rc::new(TextDocument::new(ed_cx, "")));
                });
            }
        });
        
        create_effect(move |_| {
            info!("->> effect: switch doc");
            match active_room.get() {
                Some(room) => {
                    if let Some(doc) = self.docs.with_untracked(|d| d.get(&room.id).cloned()) {
                        self.active_doc.set(Some(doc.clone()));
                        doc_signal.set(doc);
                    }
                },
                None => {}
            }
        });
        
        create_effect(move |_| {
            info!("->> effect: create msg");
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
                    let rooms_map = rooms.get_untracked();
                    let room = rooms_map.get(&active_room.idx).unwrap();
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
                // -- Save it as chunk
                let is_success = rooms.with_untracked(|rooms| {
                    if let Some(room) = rooms.get(&active_room.idx) {
                        room.msgs.update(|r| r.append_new_msg(new_msg));
                        room.update_msg_count();
                        trace!("Inserted new Msg to room");
                        return true
                    } else {
                        error!("Did not insert new Msg to room");
                        false
                    }
                });
                if is_success {
                    msg_event.set(MsgEvent::NewFor(active_room.id));
                    doc_signal.with_untracked(|doc| {
                        let text_len = doc.text().len();
                        doc.edit_single(Selection::region(0, text_len), "", EditType::DeleteSelection);
                    });
                }
            }
        });
            
        stack((container(text_editor)
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
        ).into_any()   
    }
}

// MARK: ed_toolbar

pub fn editor_toolbar_view() -> impl IntoView {
    let send_msg = use_context::<Trigger>().unwrap();
    stack((
        v_stack((
            button("Send").action(move || send_msg.notify()),
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