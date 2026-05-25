mod checkin;
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

    // CLI: `cosmic-applet-todo configure-checkin` reads the config and applies
    // the current daily_checkin_time / daily_checkin_enabled to systemd.
    // Used by `just install-user` after install and by power users who edit
    // the config file directly.
    if std::env::args().nth(1).as_deref() == Some("configure-checkin") {
        match checkin::apply(&config.daily_checkin_time, config.daily_checkin_enabled) {
            Ok(()) => {
                println!(
                    "Applied check-in: time={} enabled={}",
                    config.daily_checkin_time, config.daily_checkin_enabled
                );
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("configure-checkin failed: {e}");
                std::process::exit(1);
            }
        }
    }

    cosmic::applet::run::<Window>(config)
}
