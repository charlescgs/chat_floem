#![allow(unused)]
use floem::{event::{Event, EventListener}, keyboard::{Key, NamedKey}, kurbo::Size, reactive::create_effect, style::AlignSelf, style_class, taffy::{AlignItems, JustifyContent, Position}, views::{dyn_view, VirtualDirection, VirtualItemSize, VirtualVector}, IntoView, View};
use floem::views::{button, stack_from_iter};
use floem::taffy::{style_helpers::{minmax, TaffyGridLine}, AlignContent, FlexDirection, FlexWrap, GridPlacement, GridTrackRepetition, LengthPercentage, Line, MaxTrackSizingFunction, MinTrackSizingFunction, TrackSizingFunction};
use floem::window::{WindowConfig, WindowId};
use floem::views::{container, dyn_container, h_stack, label, scroll, stack, text_input, v_stack, virtual_list, Decorators, TextInput};
use floem::style::{Background, Style, Transition};
use floem::reactive::{create_rw_signal, create_signal, provide_context, use_context, RwSignal};
use floem::peniko::Color;
use im::Vector;

pub mod element;

pub const SIDEBAR_WIDTH: f64 = 150.0;
pub const TOPBAR_HEIGHT: f64 = 35.0;
pub const BG: Color = Color::rgb8(180, 180, 180);
pub const BUTTON_HOVER: Color = Color::rgb8(250, 250, 0);
pub const BUTTON_ACTIVE: Color = Color::rgb8(250, 0, 0);
style_class!(pub Button);



fn main() {
    let window_config = WindowConfig::default()
        .size(Size {
            width: 900.0,
            height: 800.0,
        })
        .resizable(true)
        .title("chat");

    let button = Style::new()
        .background(Color::PALE_GOLDENROD)
        .active(|s| s.background(Color::RED).color(Color::WHITE))
        .border(2.0)
        .transition(Background, Transition::linear(0.3))
        .outline_color(Color::YELLOW)
        .focus_visible(|s| s.outline(10.0).outline_color(Color::GREEN_YELLOW.with_alpha_factor(0.3)))
        .border_color(Color::BLACK)
        .hover(|s| s.background(Color::YELLOW))
        .padding(20.0)
        .border_radius(8.0)
        .margin(6.0);

    let debug_theme = Style::new()
        .background(Color::LIGHT_GRAY)
        .class(Button, move |_| button)
        .font_size(20.0);
    
    floem::Application::new()
        .window(new_app_window, Some(window_config))
        .run()
}




#[derive(Clone)]
enum Sig {
    Main,
    Flex,
    Grid,
    Gridv2,
    Simple
}


pub fn new_app_window(id: WindowId) -> impl IntoView {
    let sig = create_rw_signal(Sig::Main);
    provide_context(sig);

    let view = dyn_view(move || match sig.get() {
        Sig::Main => main_example().into_any(),
        Sig::Flex => flex_example().into_any(),
        Sig::Grid => grid_example().into_any(),
        Sig::Gridv2 => grid_examplev2().into_any(),
        Sig::Simple => simple().into_any()
    }).style(|s|s.size_full());

    let id = view.id();
    view.on_event_stop(EventListener::KeyUp, move |e| {
        if let Event::KeyUp(e) = e {
            if e.key.logical_key == Key::Named(NamedKey::F11) {
                id.inspect();
            }
        }
    })
}


fn main_example() -> impl IntoView {
    let sig = use_context::<RwSignal<Sig>>().unwrap();

    v_stack((
        button(||"Flex").on_click_stop(move |_| {
            sig.set(Sig::Flex)
        }),
        button(||"Grid").on_click_stop(move |_| {
            sig.set(Sig::Grid)
        }),
        button(||"Gridv2").on_click_stop(move |_| {
            sig.set(Sig::Gridv2)
        }),
        button(||"Simple").on_click_stop(move |_| {
            sig.set(Sig::Simple)
        }),
    )).style(|s|s.width_full().height_full().align_content(AlignContent::Center))
}


pub fn flex_example() -> impl IntoView {
    let mut iter_stack = vec![];
    for each in 0..5 {
        iter_stack.push(label(move ||format!("Container {}", each + 1)).style(|s|s.font_bold().font_size(30.0)
            // .flex_grow(1.0)
            // .flex_shrink(1.0)
            // .flex_basis(400)
            .padding(30).border(1).border_color(Color::BLACK))
        )
    }
    v_stack((
        container(button(||"go back").on_click_stop(move |_| {
            let sig = use_context::<RwSignal<Sig>>().unwrap();
            sig.set(Sig::Main)
        })),
        // stack_from_iter(
        stack(
            // iter_stack.into_iter()
            (
            label(||"Container 1").style(|s|s.font_bold().font_size(20.0)
                .align_self(AlignItems::Stretch)
                // .flex_basis(100)
                // .flex_grow(0.2)
                // .flex_shrink(1.0)
                .padding(30).border(1).border_color(Color::BLACK)),
            label(||"Container 2").style(|s|s.font_bold().font_size(20.0)
            .align_self(AlignItems::FlexStart)
                // .flex_basis(100)
                // .flex_grow(0.1)
                // .flex_shrink(1.0)
                .padding(30).border(1).border_color(Color::BLACK)),
            label(||"Container 3").style(|s|s.font_bold().font_size(20.0)
                // .flex_basis(100)
                // .flex_grow(0.5)
                // .flex_shrink(1.0)
                .padding(30).border(1).border_color(Color::BLACK)),
            label(||"Container 4").style(|s|s.font_bold().font_size(20.0)
                .align_self(AlignItems::Center)
                // .flex_basis(100)
                // .flex_grow(0.0)
                // .flex_shrink(1.0)
                .padding(30).border(1).border_color(Color::BLACK))
        )
        ).style(|s|s
            .flex()
            // .flex_direction(FlexDirection::Row)
            .align_items(AlignItems::FlexEnd)
            .min_height(200)
            // .width(800)
            // .flex_wrap(FlexWrap::Wrap)
            
            // .justify_content(JustifyContent::)
        )
    )).style(|s|s.width_full().height_full())
}


fn simple() -> impl IntoView {
    let room_view = Vector::from(vec!["asdsdasda".to_string(), "tertrte".to_string(), "werweewg".to_string(), "ertrtrtret".to_string(), "rtwertertwe".to_string(), "werwerwerwer".to_string(), "etwerrtwerwer".to_string(), "tyutyutyth".to_string(), "yujkhngcvb".to_string(), "gZsdczsxcasc".to_string()]);
    let room_view = create_rw_signal(room_view);

    let mut selected_room = create_rw_signal(String::new());
    
    let room_msgs = Vector::from(vec![
        "asdsdsdasda Msg_autor".to_string(), "tertrte Msg_autor".to_string(), "werweewg Msg_autor".to_string(), "ertrtrtret Msg_autor".to_string(), "rtwertertwe Msg_autor".to_string(), "werwerwerwer Msg_autor".to_string(), "etwerrtwerwer Msg_autor".to_string(), "tyutyutyth Msg_autor".to_string(), "yujkhngcvb Msg_autor".to_string(), "gZsdczsxcasc Msg_autor".to_string(),
        "teqawdrtrte Msg_autor".to_string(), "werhtyweewg Msg_autor".to_string(), "ertrtrarret Msg_autor".to_string(), "rtwertesdrtwe Msg_autor".to_string(), "werwerweryjwer Msg_autor".to_string(), "etwerrwerwefttwerwer Msg_autor".to_string(), "tyutyuyutyth Msg_autor".to_string(), "yujkhngcyuvb Msg_autor".to_string(), "gZsdczaefsxcasc Msg_autor".to_string()
    ]);
    let room_msgs = create_rw_signal(room_msgs);
    
    create_effect(move |_| {
        let active = selected_room.get();
        if active == "asdsdasda" {
            // room_msgs.
        }
    });
    
    v_stack((
        h_stack((
            button(||"go back").on_click_stop(move |_| {
                let sig = use_context::<RwSignal<Sig>>().unwrap();
                sig.set(Sig::Main)
            }),
            button(||"Menu").on_click_stop(move |_| {
                let sig = use_context::<RwSignal<Sig>>().unwrap();
                sig.set(Sig::Main)
            }),
            button(||"Preferences").on_click_stop(move |_| {
                let sig = use_context::<RwSignal<Sig>>().unwrap();
                sig.set(Sig::Main)
            }),
            button(||"About").on_click_stop(move |_| {
                let sig = use_context::<RwSignal<Sig>>().unwrap();
                sig.set(Sig::Main)
            }),
        )).style(|s|s
            .width_full()
            .height(40.0)
            .align_items(AlignItems::Center)
            .justify_content(AlignContent::Start)
            .margin(3)
            .gap(5, 0)
        ),
        h_stack((
            scroll(
                virtual_list(
                    VirtualDirection::Vertical,
                    VirtualItemSize::Fixed(Box::new(|| 120.0)),
                    move || room_view.get(),
                    move |x| x.clone(),
                    move |x| {
                        let x = x.clone();
                        let y = x.clone();
                        button(move || x.clone()).style(|s|s
                            .border(1)
                            .border_color(Color::BLACK)
                            .border_radius(5)
                            .size(238, 110)
                            .margin_bottom(5)
                        ).on_click_stop(move |_| {
                            selected_room.set(y.clone())
                        })
                    }
                )
            ).style(|s|s
                .height_full()
                .width(250)
                .font_size(15.0)
                .border(1)
                .border_color(Color::BLACK)
                .border_radius(5)
                .padding(5)
            ),
            stack((
                label(||"Text field").style(|s|s
                    .font_bold()
                    .font_size(20.0)
                    .border_radius(5)
                    .padding(15)
                    .border(1)
                    .border_color(Color::BLACK)
                    .height_pct(15.0)
                    .width_full()
                ),
                scroll(
                    virtual_list(
                        VirtualDirection::Vertical,
                        VirtualItemSize::Fixed(Box::new(|| 65.0)),
                        move || room_msgs.get().enumerate(),
                        move |msg| msg.clone(),
                        |(index, msg)| {
                            if index % 2 == 0 {
                                stack((
                                    label(move || msg.clone()),
                                    label( ||"another label"),
                                    label( ||"anotation")
                                )).style(|s|s
                                    .flex_col()
                                    .align_self(AlignItems::End)
                                    .align_items(AlignItems::End)
                                    // .justify_content(JustifyContent::FlexStart)
                                    .border(1)
                                    .border_color(Color::BLACK)
                                    .border_radius(5)
                                    .padding(5)
                                )
                            } else {
                                // let (x,y) = msg.split_once(' ').unwrap();
                                stack((
                                    label(move || msg.clone()),
                                    label( || "label to view")
                                )).style(|s|s
                                    .flex_col()
                                    .align_self(AlignItems::Start)
                                    .align_items(AlignItems::Start)
                                    .border(1)
                                    .border_color(Color::BLACK)
                                    .border_radius(5)
                                    .padding(5)
                                )
                            }
                        }
                    ).style(|s|s // virtual list
                        .flex()
                        .flex_direction(FlexDirection::Column)
                        .min_width_full()
                        .padding(5)
                    )
                ).style(|s|s // scroll
                    .border_radius(5)
                    .height_pct(85.0)
                    .width_full()
                )
            )).style(|s| s // stack
                .flex_direction(FlexDirection::ColumnReverse)
                .width_full()
                .height_full()
            )
        )).style(|s|s.size_full())
    )).style(|s|s.size_full())
}



pub fn grid_example() -> impl IntoView {
    let mut rooms = vec![];
    for each in 0..7 {
        rooms.push(label(move ||format!("Container {}", each + 1)).style(|s|s
                .font_bold()
                .font_size(30.0)
                .padding(30).border(1).border_color(Color::BLACK))
            )
    }
    v_stack((
        container(button(||"go back").on_click_stop(move |_| {
            let sig = use_context::<RwSignal<Sig>>().unwrap();
            sig.set(Sig::Main)
        })),
        stack((
            // stack_from_iter(stack.into_iter())
            // stack((
                label(||"Room list").style(|s|s
                    .font_bold()
                    .font_size(30.0)
                    .border(1)
                    .border_color(Color::DARK_GREEN)
                    .border_radius(5)
                    .padding(30)
                    .align_self(AlignItems::Stretch)
                    .grid_column(Line {
                        start: GridPlacement::from_line_index(1),
                        end: GridPlacement::Auto
                    })
                    .grid_row(Line {
                        start: GridPlacement::from_line_index(1),
                        end: GridPlacement::Span(2)
                    })
                ),
            label(||"Messages").style(|s|s.font_bold().font_size(30.0)
                .padding(30).border(1).border_color(Color::BLACK).border_radius(5)
                .grid_column(Line {
                    start: GridPlacement::from_line_index(2),
                    end: GridPlacement::Auto
                })
                .grid_row(Line {
                    start: GridPlacement::from_line_index(1),
                    end: GridPlacement::Auto
                })
            ),
            label(||"Text field").style(|s|s.font_bold().font_size(20.0).border_radius(5)
                .padding(15).border(1).border_color(Color::BLACK)
                .grid_column(Line {
                    start: GridPlacement::from_line_index(2),
                    end: GridPlacement::Auto
                })
                .grid_row(Line {
                    start: GridPlacement::from_line_index(2),
                    end: GridPlacement::Auto
                })
                .align_self(AlignItems::Stretch)
                // .justify_self(AlignItems::Stretch)
            )
            // label(||"Container 5").style(|s|s.font_bold().font_size(30.0)
            // .padding(30).border(1).border_color(Color::BLACK))
            ))
        .style(|s|s
            .grid()
            // .grid_auto_columns(vec![
            //     minmax(
            //         // MinTrackSizingFunction::Auto,
            //         MinTrackSizingFunction::Fixed(LengthPercentage::Length(100.0)),
            //         MaxTrackSizingFunction::Auto
            //     )
            // ])
            // .grid_auto_rows(vec![
            //     minmax(
            //         // MinTrackSizingFunction::Auto,
            //         MinTrackSizingFunction::Fixed(LengthPercentage::Length(100.0)),
            //         MaxTrackSizingFunction::Auto
            //     ),
            // ])
            .grid_template_columns(vec![
                TrackSizingFunction::Repeat(
                    GridTrackRepetition::Count(2),
                    vec![
                        minmax(
                            MinTrackSizingFunction::Fixed(LengthPercentage::Length(250.0)),
                            MaxTrackSizingFunction::Fraction(1.0)
                        ),
                        minmax(
                            MinTrackSizingFunction::Fixed(LengthPercentage::Length(650.0)),
                            MaxTrackSizingFunction::Fraction(1.0)
                        ),
                    ]
                ),
                // TrackSizingFunction::Single(
                //     minmax(
                //         MinTrackSizingFunction::Auto,
                //         MaxTrackSizingFunction::Auto
                //     )
                // )
            ])
            .grid_template_rows(vec![
                TrackSizingFunction::Repeat(
                    GridTrackRepetition::Count(2),
                    vec![
                        minmax(
                            MinTrackSizingFunction::Fixed(LengthPercentage::Length(650.0)),
                            MaxTrackSizingFunction::Fraction(1.0)
                        ),
                        minmax(
                            MinTrackSizingFunction::Fixed(LengthPercentage::Length(60.0)),
                            MaxTrackSizingFunction::Fraction(1.0)
                        )
                    ]
                ),
                // TrackSizingFunction::Single(
                //     minmax(
                //         MinTrackSizingFunction::Auto,
                //         MaxTrackSizingFunction::Auto
                //     )
                // )
            ])
        //     // .grid_auto_rows(vec![
        //     //     minmax(
        //     //         // MinTrackSizingFunction::Auto,
        //     //         MinTrackSizingFunction::Fixed(LengthPercentage::Length(100.0)),
        //     //         MaxTrackSizingFunction::Auto
        //     //     )
        //     // ])
            // .gap(5, 5)
        )
        // .style(|s|s.height_full())
    )).style(|s|s.max_height(800).max_width(900))
}


pub fn grid_examplev2() -> impl IntoView {
    let mut rooms = vec![];
    for each in 0..7 {
        rooms.push(label(move ||format!("Container {}", each + 1)).style(|s|s
                .font_bold()
                .font_size(30.0)
                .padding(30).border(1).border_color(Color::BLACK))
            )
    }
    v_stack((
        container(button(||"go back").on_click_stop(move |_| {
            let sig = use_context::<RwSignal<Sig>>().unwrap();
            sig.set(Sig::Main)
        })).style(|s|s.width_full().height(40.0)),
        stack((
            // stack_from_iter(stack.into_iter())
            // stack((
                label(||"Room list").style(|s|s
                    .font_bold()
                    .font_size(30.0)
                    .border(1)
                    .border_color(Color::DARK_GREEN)
                    .border_radius(5)
                    .padding(30)
                    // .align_self(AlignItems::Stretch)
                    .grid_column(Line {
                        start: GridPlacement::from_line_index(1),
                        end: GridPlacement::Span(2)
                    })
                    .grid_row(Line {
                        start: GridPlacement::from_line_index(1),
                        end: GridPlacement::Span(4)
                    })
                ),
            label(||"Messages").style(|s|s.font_bold().font_size(30.0)
                .padding(30).border(1).border_color(Color::BLACK).border_radius(5)
                .grid_column(Line {
                    start: GridPlacement::from_line_index(3),
                    end: GridPlacement::Span(4)
                })
                .grid_row(Line {
                    start: GridPlacement::from_line_index(1),
                    end: GridPlacement::Span(4)
                })
            ),
            label(||"Text field").style(|s|s.font_bold().font_size(20.0).border_radius(5)
                .padding(15).border(1).border_color(Color::BLACK)
                .grid_column(Line {
                    start: GridPlacement::from_line_index(3),
                    end: GridPlacement::Span(4)
                })
                .grid_row(Line {
                    start: GridPlacement::from_line_index(4),
                    end: GridPlacement::Span(1)
                })
                // .align_self(AlignItems::Stretch)
                // .justify_self(AlignItems::Stretch)
            )
            // label(||"Container 5").style(|s|s.font_bold().font_size(30.0)
            ))
        .style(|s|s
            .grid()
            .grid_template_columns(vec![
                TrackSizingFunction::Repeat(
                    GridTrackRepetition::Count(6),
                    vec![
                        minmax(
                            MinTrackSizingFunction::Fixed(LengthPercentage::Length(150.0)),
                            MaxTrackSizingFunction::Auto
                        ),
                        // minmax(
                        //     MinTrackSizingFunction::Auto,
                        //     MaxTrackSizingFunction::Fraction(2.0)
                        // ),
                        // minmax(
                        //     MinTrackSizingFunction::Auto,
                        //     MaxTrackSizingFunction::Fraction(2.0)
                        // ),
                        // minmax(
                        //     MinTrackSizingFunction::Auto,
                        //     MaxTrackSizingFunction::Fraction(1.0)
                        // ),
                    ]
                )
            ])
            .grid_template_rows(vec![
                TrackSizingFunction::Repeat(
                    GridTrackRepetition::Count(4),
                    vec![
                        minmax(
                            MinTrackSizingFunction::Fixed(LengthPercentage::Length(190.0)),
                            MaxTrackSizingFunction::Fraction(1.0)
                        ),
                        // minmax(
                        //     MinTrackSizingFunction::Fixed(LengthPercentage::Length(150.0)),
                        //     MaxTrackSizingFunction::Fraction(1.0)
                        // ),
                        // minmax(
                        //     MinTrackSizingFunction::Fixed(LengthPercentage::Length(150.0)),
                        //     MaxTrackSizingFunction::Fraction(1.0)
                        // ),
                        // minmax(
                        //     MinTrackSizingFunction::Fixed(LengthPercentage::Length(150.0)),
                        //     MaxTrackSizingFunction::Fraction(1.0)
                        // )
                    ]
                )
            ])
            // .gap(2, 2)
            // .margin(1)
        )
    )).style(|s|s
        // .max_height(800).max_width(900)
        // .size_pct(900.0, 770.0)
        .max_height_full()
        .max_width_full()
    )
}
