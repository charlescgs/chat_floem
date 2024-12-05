use std::fs::OpenOptions;
use std::io::BufReader;
use std::ops::{DivAssign, MulAssign};

use serde::{Deserialize, Serialize};
use tracing_lite::error;
use floem::action::{inspect, set_window_scale};
use floem::event::{Event, EventListener, EventPropagation};
use floem::keyboard::{Key, Modifiers, NamedKey};
use floem::{prelude::*, Application};
use floem::kurbo::{Point, Size};
use floem::reactive::provide_context;
use floem::window::{Theme, WindowConfig};


const CONFIG_PATH: &str = "cc.txt";


/// Configuration struct for the chat client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    theme: Theme,
    language: Lang, // TODO!
    position: Point,
    size: Size,
    scale: f64
}

impl ChatConfig {
    pub fn fetch() -> Self {
        let config_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(CONFIG_PATH);
        match config_file {
            Ok(cf) => {
                let reader = BufReader::new(cf);
                serde_json::from_reader(reader).unwrap_or_default()
            },
            Err(e) => { error!("Failed to open config file: {e}"); Self::default() }
        }
    }
    
    pub fn save_to_file(&self) -> Option<()> {
        let config_file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .create(true)
            .open(CONFIG_PATH);
        match config_file {
            Ok(cf) => {
                match serde_json::to_writer_pretty(cf, self) {
                    Ok(_) => Some(()),
                    Err(e) => { error!("Failed to save config file: {e}"); None }
                }
            },
            Err(e) => { error!("Failed to open config file: {e}"); None }
        }
    }
}


impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            position: Point { x: 500., y: 500. },
            size: Size {
                width: 700.,
                height: 520.
            },
            scale: 1.,
            language: Lang::English
        }
    }
}


/// Launch application with reactive config and window resizing.
pub fn launch_with_config<V: IntoView + 'static>(app_view: impl FnOnce() -> V + 'static) {
    // -- Fetch application config of apply default one
    let config = RwSignal::new(ChatConfig::fetch());
    // -- Save it as a context into floem runtime
    provide_context(config);
    // -- Provide reactive way to update config and save it to file
    // create_updater(move || config.get(), |cf|{ cf.save_to_file(); });
    // -- Construct window settings from config with reactive updates (.with())
    let win_config = WindowConfig::default()
        .size(config.with(|cf| cf.size))
        .position(config.with(|cf| cf.position));
    // -- Construct application window with ability to change its
    //    dimensions via keyboard shortcuts and save all changes in the config
    Application::new()
        .window(move |_| {
            // -- Provide reactive way to update window scale
            set_window_scale(config.with(|cf| cf.scale));
            app_view()
                .keyboard_navigable()
                // -- Make window 10% bigger
                .on_key_down(
                    Key::Character("=".into()),
                    |m| m == Modifiers::CONTROL,
                    move |_| {
                        config.update(|cf| { cf.scale.mul_assign(1.1); set_window_scale(cf.scale); });
                    })
                // -- Make scale 10% smaller
                .on_key_down(
                    Key::Character("-".into()),
                    |m| m == Modifiers::CONTROL,
                    move |_| {
                        config.update(|cf| { cf.scale.div_assign(1.1); set_window_scale(cf.scale); });
                    })
                // -- Make scale back to default
                .on_key_down(
                    Key::Character("0".into()),
                    |m| m == Modifiers::CONTROL,
                    move |_| {
                        config.update(|cf| { cf.scale = 1.; set_window_scale(cf.scale); });
                    })
                // -- Open inspector on F11 key
                .on_key_down(
                    Key::Named(NamedKey::F11),
                    |m| m.is_empty(),
                    |_| {
                        inspect();
                    },
                )
                // -- Update position in config on move end
                .on_event_stop(
                    EventListener::WindowMoved,
                    move |ev|
                        if let Event::WindowMoved(pos) = ev {
                            config.update(|cf| cf.position = *pos)
                        }
                )
                // -- Update size in config on resize end
                .on_event_stop(
                    EventListener::WindowResized,
                    move |ev|
                        if let Event::WindowResized(size) = ev {
                            config.update(|cf| cf.size = *size)
                        }
                )
                .on_event(
                    EventListener::WindowClosed,
                    move |ev| {
                        if let Event::WindowClosed = ev {
                            config.get().save_to_file();
                        }
                        EventPropagation::Continue
                    }
                )
        },
        Some(win_config)
    ).run();
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Lang {
    English,
    Polish
}