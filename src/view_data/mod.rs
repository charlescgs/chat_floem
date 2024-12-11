use std::time::Duration;

use floem::action::exec_after;
use floem::action::TimerToken;
use floem::reactive::Trigger;
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


/// Debounce an action
///
/// This tracks a trigger will run action only once an **uninterrupted** duration has passed.
pub fn trigger_debounce_action(trigger: Trigger, duration: Duration, action: impl Fn() + Clone + 'static) {
    floem::reactive::create_stateful_updater(
        move |prev_opt: Option<Option<TimerToken>>| {
            trigger.track();
            let execute = true;
            (execute, prev_opt.and_then(|timer| timer))
        },
        move |execute, prev_timer| {
            // Cancel the previous timer if it exists
            if let Some(timer) = prev_timer {
                timer.cancel();
            }
            let timer_token = if execute {
                let action = action.clone();
                Some(exec_after(duration, move |_| {
                    action();
                }))
            } else {
                None
            };
            timer_token
        },
    );
}