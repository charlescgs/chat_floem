use floem::prelude::*;
use floem::reactive::{create_effect, use_context, Trigger};
use floem::taffy::prelude::TaffyGridLine;
use floem::taffy::{GridPlacement, Line};
use im::Vector;
use tracing_lite::info;

use crate::view_data::room::RoomTabIdx;
use crate::view_data::session::APP;
use crate::view_data::MsgEvent;



// pub struct RoomsView {
//     rooms_triggers: Vector<(usize, Trigger)>
// }


/// This function:
/// - [x] constructs list of the rooms
/// - [x] updates that list on changes
/// - [ ] communicate with msgs
/// - [ ] communicate with backend
pub fn rooms_view_v2() -> impl IntoView {
    info!("->> rooms_view");
    // -- Needed elements
    let rooms = APP.with(|a| a.rooms);
    let active = APP.with(|a| a.active_room);
    // let msg_event = use_context::<RwSignal<MsgEvent>>().unwrap();
    // let room_selected = Trigger::new();
    
    // -- Effect and derives needed for the view
    
    // -- View stack
    stack((
        dyn_stack(
            move || rooms.get(),
            move |(id, _)| *id,
            move |(id, room)| {
                room
            }
        ).style(|s| s
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
        ).scroll_style(|s| s.handle_thickness(6.).shrink_to_fit()),
    ))
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
    // 3. Keep in sync:
    // - process user clicks and update active tab
    // - react on last room msg
    // - react on new/changed msg and show unread status
}