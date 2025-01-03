#![allow(unused)]

use config::launch_with_config;
use floem::prelude::*;
use floem::reactive::{provide_context, Trigger};
use floem::taffy::prelude::minmax;
use floem::taffy::{
    LengthPercentage, MaxTrackSizingFunction,
    MinTrackSizingFunction, TrackSizingFunction
};
use tracing_lite::{Level, Subscriber};
use ulid::Ulid;
use util::Id;
use view_data::editor::{editor_toolbar_view, EditorViewData};
use view_data::MsgEvent;
use views::msgs::msgs_view;
use views::rooms::rooms_view;
use views::toolbar::toolbar_view;

pub mod common;
pub mod view_data;
pub mod config;
pub mod cont {
    pub mod msg;
    pub mod room;
    pub mod acc;
}
pub mod util;
pub mod views {
    pub mod msgs;
    pub mod rooms;
    pub mod toolbar;
}
pub mod chunks;

pub const SIDEBAR_WIDTH: f64 = 150.0;
pub const TOPBAR_HEIGHT: f64 = 35.0;
pub const BG: Color = Color::rgb8(180, 180, 180);
pub const BUTTON_HOVER: Color = Color::rgb8(250, 250, 0);
pub const BUTTON_ACTIVE: Color = Color::rgb8(250, 0, 0);


// MARK: MAIN

// -----------------------
fn main() {
    Subscriber::new_with_max_level(Level::TRACE);
    
    provide_context(RwSignal::new(None::<Id>));     // Msg tracker
    provide_context(RwSignal::new(MsgEvent::None)); // Msg load tracker
    provide_context(RwSignal::new(None::<Ulid>));   // New room id editor signal
    provide_context(Trigger::new());                // Msg send signal
    provide_context(RwSignal::new(false));          // Load more signal
    
    launch_with_config(app_view)
}


fn app_view() -> impl IntoView {
    stack((
        toolbar_view(),
        rooms_view(),
        msgs_view(),
        EditorViewData::new(), // OR: text_editor_view(send_msg),
        editor_toolbar_view(),
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