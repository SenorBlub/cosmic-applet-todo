use std::fs;
use std::path::PathBuf;
use std::process::Command;

const TIMER_UNIT: &str = "todo-daily-checkin.timer";
const DROP_IN_DIR: &str = "systemd/user/todo-daily-checkin.timer.d";
const DROP_IN_FILE: &str = "override.conf";

/// Parse an `HH:MM` string into hour (0–23) and minute (0–59).
/// Single-digit hours like `8:00` are accepted.
pub fn parse_hhmm(s: &str) -> Result<(u8, u8), String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err("empty".to_string());
    }
    let (h, m) = trimmed
        .split_once(':')
        .ok_or_else(|| "expected HH:MM".to_string())?;
    let hour: u8 = h.parse().map_err(|_| format!("invalid hour `{h}`"))?;
    let minute: u8 = m.parse().map_err(|_| format!("invalid minute `{m}`"))?;
    if hour > 23 {
        return Err(format!("hour {hour} out of range (0–23)"));
    }
    if minute > 59 {
        return Err(format!("minute {minute} out of range (0–59)"));
    }
    Ok((hour, minute))
}

fn drop_in_path() -> PathBuf {
    dirs::config_dir()
        .map(|c| c.join(DROP_IN_DIR).join(DROP_IN_FILE))
        .unwrap_or_else(|| PathBuf::from(DROP_IN_FILE))
}

fn write_drop_in(hour: u8, minute: u8) -> Result<(), String> {
    let path = drop_in_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    // The empty `OnCalendar=` resets accumulated values from the base unit
    // before re-setting; without it systemd appends a second trigger.
    let body = format!("[Timer]\nOnCalendar=\nOnCalendar=*-*-* {hour:02}:{minute:02}:00\n");
    fs::write(&path, body).map_err(|e| format!("write {}: {e}", path.display()))
}

fn run_systemctl(args: &[&str]) -> Result<(), String> {
    let mut full_args = vec!["--user"];
    full_args.extend_from_slice(args);
    let status = Command::new("systemctl")
        .args(&full_args)
        .status()
        .map_err(|e| format!("systemctl: {e}"))?;
    if !status.success() {
        return Err(format!("systemctl {} failed", args.join(" ")));
    }
    Ok(())
}

/// Apply the timer configuration: write the drop-in, reload systemd, and
/// enable or disable the timer accordingly. Idempotent.
pub fn apply(time: &str, enabled: bool) -> Result<(), String> {
    let (h, m) = parse_hhmm(time)?;
    write_drop_in(h, m)?;
    run_systemctl(&["daemon-reload"])?;
    if enabled {
        run_systemctl(&["enable", "--now", TIMER_UNIT])?;
    } else {
        run_systemctl(&["disable", "--now", TIMER_UNIT])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_time() {
        assert_eq!(parse_hhmm("08:00"), Ok((8, 0)));
        assert_eq!(parse_hhmm("23:59"), Ok((23, 59)));
        assert_eq!(parse_hhmm("00:00"), Ok((0, 0)));
    }

    #[test]
    fn accepts_single_digit_hour() {
        assert_eq!(parse_hhmm("8:00"), Ok((8, 0)));
    }

    #[test]
    fn trims_surrounding_whitespace() {
        assert_eq!(parse_hhmm("  16:30  "), Ok((16, 30)));
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(parse_hhmm("24:00").is_err());
        assert!(parse_hhmm("12:60").is_err());
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_hhmm("").is_err());
        assert!(parse_hhmm("abc").is_err());
        assert!(parse_hhmm("8").is_err());
        assert!(parse_hhmm("08:00:00").is_err());
        assert!(parse_hhmm("8:0a").is_err());
    }
}
