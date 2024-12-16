use std::time::Duration;

use floem::action::exec_after;
use floem::action::TimerToken;
use floem::reactive::Trigger;
use tracing_lite::debug;
use tracing_lite::info;
use ulid::Ulid;


pub mod room;
pub mod msg;
pub mod editor;
pub mod session;


#[derive(Clone, Debug)]
pub enum MsgEvent {
    None,
    /// Brand new msg for the provided room. 
    NewFor(Ulid),
    NewManyFor(Ulid),
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



/// This tracks a trigger, that will run action only on second trigger.
pub fn run_on_second_trigger(trigger: Trigger, action: impl Fn() + 'static) {
    floem::reactive::create_stateful_updater(
        move |previous_trigger: Option<bool>| {
            trigger.track();
            let execute = previous_trigger
                .map(|prev_run| !prev_run)
                .unwrap_or(true);
        ((), execute)
    },
        move |_, execute| {
            if execute {
                info!("fn: run_on_second_trigger: executing action!");
                action();
            }
            execute
        }
    );
}