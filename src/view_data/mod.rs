use ulid::Ulid;


pub mod room;
pub mod msg;
pub mod editor;
pub mod session;


#[derive(Clone)]
pub enum MsgEvent {
    None,
    /// Brand new msg for the provided room. 
    NewFor(Ulid),
    /// Updated msg for the given room.
    UpdatedFor {
        room: Ulid,
        msg: Ulid
    },
    /// Deleted msg for the given room.
    Deleted {
        room: Ulid,
        msg: Ulid
    }
}