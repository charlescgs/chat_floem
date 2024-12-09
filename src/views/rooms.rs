use floem::prelude::*;
use floem::reactive::create_effect;
use floem::taffy::prelude::TaffyGridLine;
use floem::taffy::{GridPlacement, Line};
use tracing_lite::info;

use crate::view_data::room::RoomTabIdx;
use crate::view_data::session::APP;



// pub struct RoomsView {
//     rooms: 
// }


/// Cunstructs and keeps in sync all rooms list.
pub fn rooms_view_v2() -> impl IntoView {
    info!("->> rooms_view");
    // 1. Get rooms.
    let rooms = APP.with(|a| a.rooms);
    let active = APP.with(|a| a.active_room);
    // 2. Show them as list.
    // let room_selected = Trigger::new();

    // create_effect(move |_| {

    // });
    
    stack((
        dyn_stack(
            move || rooms.get(),
            move |(id, _)| *id,
            move |(id, room)| {
                room.get()
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