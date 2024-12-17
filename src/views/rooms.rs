use floem::prelude::*;
use floem::reactive::create_effect;
use floem::taffy::prelude::TaffyGridLine;
use floem::taffy::{GridPlacement, Line};
use tracing_lite::{debug, info};

use crate::view_data::session::APP;



// pub struct RoomsView {
//     rooms_triggers: Vector<(usize, Trigger)>
// }


/// This function:
/// - [x] constructs list of the rooms
/// - [x] updates that list on changes
/// - [ ] react on new/changed msg and show unread status
/// - [ ] communicate with msgs
/// - [ ] communicate with backend
pub fn rooms_view_v2() -> impl IntoView {
    info!("->> rooms_view");
    // -- Needed elements
    let rooms = APP.with(|a| a.rooms);
    let active = APP.with(|a| a.active_room);
    
    // -- Effects and derives needed for the view
    create_effect(move |_| {
        debug!("== effect(is_active run)");
        // active.track();
        active.with(|act| {
            if let Some(active) = act {
                rooms.with_untracked(|rooms| {
                    for (idx, room) in rooms {
                        if *idx != active.idx {
                            println!("changed to false: {}", room.idx());
                            room.is_active.with_untracked(|cell| cell.set(false));
                        }
                    }
                });
            }
        })
    });
    
    // -- View stack
    stack((
        dyn_stack(
            move || rooms.get(),
            move |(id, _)| *id,
            move |(_, room)| {
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
}