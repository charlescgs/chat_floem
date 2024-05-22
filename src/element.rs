use floem::{reactive::RwSignal, style::Style, style_class, views::{button, Decorators, VirtualVector}, IntoView};


// pub struct SimpleButton;
style_class!(pub SimpleButton);

pub fn simple_button(label: String) -> impl IntoView {
    button(move || label.clone()).style(|s|s
        .class(SimpleButton, button_style)
    )
}

pub fn button_style(s: Style) -> Style {
    todo!()
}


pub enum UpdateRoomView {

}



pub struct RoomView {
    room_id: u64,
    last_msg: String,
    last_msg_author: String,
    // TODO: author_picture: Bytes
    msg_date: String,
    // update: RwSignal<UpdateRoomView>
}



// impl<T> VirtualVector<T> for RoomView {
//     fn total_len(&self) -> usize {
//         todo!()
//     }

//     fn slice(&mut self, range: std::ops::Range<usize>) -> impl Iterator<Item = T> {
//         todo!()
//     }
// }

