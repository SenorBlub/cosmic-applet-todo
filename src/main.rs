mod config;
mod storage;
mod window;

use crate::config::Config;
use crate::window::Window;

fn main() -> cosmic::iced::Result {
    let env = env_logger::Env::default()
        .filter_or("COSMIC_APPLET_TODO_LOG", "warn")
        .write_style_or("COSMIC_APPLET_TODO_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let config = Config::load();
    cosmic::applet::run::<Window>(config)
}
