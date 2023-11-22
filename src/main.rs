#![allow(unused)]
use floem::peniko::Color;
use floem::reactive::create_signal;
use floem::style::Position;
use floem::widgets::button;
use floem::views::{Decorators, stack, label, scroll, virtual_list, container, h_stack, v_stack};
use floem::kurbo::Size;
use floem::view::View;
use floem::window::WindowConfig;
use im::Vector;

pub const SIDEBAR_WIDTH: f64 = 150.0;
pub const TOPBAR_HEIGHT: f64 = 35.0;



fn app_view() -> impl View {
    let accounts: im::Vector<usize> = (1..150).collect();
    let (accounts, set_accounts) = create_signal(accounts);

    let top_bar = label(|| String::from("TOP BAR"))
        .style(|s| s.padding(10.0).width_full().height(TOPBAR_HEIGHT));

    let side_bar = scroll({
        virtual_list(
            floem::views::VirtualListDirection::Vertical,
            floem::views::VirtualListItemSize::Fixed(Box::new(||22.0)),
            move || accounts.get(),
            move |item| *item,
            move |item| {
                label(move || item.to_string()).style(move |s| {
                    s.padding(60.0)
                    .padding_top(30.0)
                    .padding_bottom(30.0)
                    .width(SIDEBAR_WIDTH)
                    .items_start()
                    .border(1.0)
                    .border_radius(10.0)
                    .border_color(Color::rgb8(205, 205, 205))
                })
            }
        ).style(|s|s.flex_col().width(SIDEBAR_WIDTH - 1.0))
    }).style(|s|s.width(SIDEBAR_WIDTH).border_right(1.0).border_top(1.0).border_color(Color::rgb8(205, 205, 205)).border_radius(10.0));

    let chat_window = scroll(
        container(label(move || String::from("Main Window")).style(|s|s.padding(10.0)))
            .style(|s|s.flex_col().items_start().padding_bottom(10.0)),
        ).style(|s|{
            s.flex_col()
            .flex_basis(0)
            .min_width(0)
            .flex_grow(1.0)
            .border_top(1.0)
            .border_color(Color::rgb8(205, 205, 205))
        }
    );

    let content = h_stack((side_bar, chat_window)).style(|s| {
        s.position(Position::Absolute)
            .inset_top(35.0)
            .inset_bottom(0.0)
            .width_full()
    });

    let view = v_stack((top_bar, content)).style(|s|s.width_full().height_full());
    view
}


fn main() {
    let window_config = WindowConfig::default().size(Size {
        width: 900.0,
        height: 750.0,
    }).resizable(false).title("chat");

    floem::Application::new()
        .window(move |_| app_view(), Some(window_config))
        .run()
}
