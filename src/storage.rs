use std::fs;
use std::io;
use std::path::Path;

const HEADER: &str =
    "<!-- Managed by cosmic-applet-todo. Sections: Today, This Week, Someday. -->\n\n";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Horizon {
    Today,
    Week,
    Someday,
}

impl Horizon {
    pub const ALL: [Horizon; 3] = [Horizon::Today, Horizon::Week, Horizon::Someday];

    pub fn index(self) -> usize {
        match self {
            Horizon::Today => 0,
            Horizon::Week => 1,
            Horizon::Someday => 2,
        }
    }

    pub fn heading(self) -> &'static str {
        match self {
            Horizon::Today => "Today",
            Horizon::Week => "This Week",
            Horizon::Someday => "Someday",
        }
    }

    fn from_heading(s: &str) -> Option<Self> {
        match s.trim() {
            "Today" => Some(Horizon::Today),
            "This Week" => Some(Horizon::Week),
            "Someday" => Some(Horizon::Someday),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Task {
    pub text: String,
    pub done: bool,
}

#[derive(Clone, Debug, Default)]
pub struct TodoFile {
    pub today: Vec<Task>,
    pub week: Vec<Task>,
    pub someday: Vec<Task>,
}

impl TodoFile {
    pub fn bucket(&self, h: Horizon) -> &Vec<Task> {
        match h {
            Horizon::Today => &self.today,
            Horizon::Week => &self.week,
            Horizon::Someday => &self.someday,
        }
    }

    pub fn bucket_mut(&mut self, h: Horizon) -> &mut Vec<Task> {
        match h {
            Horizon::Today => &mut self.today,
            Horizon::Week => &mut self.week,
            Horizon::Someday => &mut self.someday,
        }
    }
}

pub fn load(path: &Path) -> TodoFile {
    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return TodoFile::default(),
        Err(e) => {
            log::warn!("Failed to read {}: {}", path.display(), e);
            return TodoFile::default();
        }
    };
    parse(&raw)
}

pub fn save(path: &Path, file: &TodoFile) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let tmp_path = path.with_extension("md.tmp");
    fs::write(&tmp_path, render(file))?;
    fs::rename(&tmp_path, path)
}

fn parse(raw: &str) -> TodoFile {
    let mut file = TodoFile::default();
    let mut current: Option<Horizon> = None;
    for line in raw.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("## ") {
            current = Horizon::from_heading(rest);
            continue;
        }
        let Some(h) = current else { continue };
        if let Some(task) = parse_task_line(trimmed) {
            file.bucket_mut(h).push(task);
        }
    }
    file
}

fn parse_task_line(line: &str) -> Option<Task> {
    let rest = line.strip_prefix("- ")?;
    let mut chars = rest.chars();
    let open = chars.next()?;
    let mark = chars.next()?;
    let close = chars.next()?;
    if open != '[' || close != ']' {
        return None;
    }
    let done = match mark {
        'x' | 'X' => true,
        ' ' => false,
        _ => return None,
    };
    let text = chars.as_str().trim().to_string();
    if text.is_empty() {
        return None;
    }
    Some(Task { text, done })
}

fn render(file: &TodoFile) -> String {
    let mut out = String::new();
    out.push_str(HEADER);
    for h in Horizon::ALL {
        out.push_str("## ");
        out.push_str(h.heading());
        out.push_str("\n\n");
        for task in file.bucket(h) {
            out.push_str("- [");
            out.push(if task.done { 'x' } else { ' ' });
            out.push_str("] ");
            out.push_str(&task.text);
            out.push('\n');
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_three_sections() {
        let mut file = TodoFile::default();
        file.bucket_mut(Horizon::Today).push(Task {
            text: "Ship the applet".to_string(),
            done: false,
        });
        file.bucket_mut(Horizon::Today).push(Task {
            text: "Tea".to_string(),
            done: true,
        });
        file.bucket_mut(Horizon::Week).push(Task {
            text: "Pack for Japan".to_string(),
            done: false,
        });
        file.bucket_mut(Horizon::Someday).push(Task {
            text: "Learn Hyprland".to_string(),
            done: false,
        });

        let rendered = render(&file);
        let parsed = parse(&rendered);

        assert_eq!(parsed.today.len(), 2);
        assert_eq!(parsed.today[0].text, "Ship the applet");
        assert!(!parsed.today[0].done);
        assert!(parsed.today[1].done);
        assert_eq!(parsed.week[0].text, "Pack for Japan");
        assert_eq!(parsed.someday[0].text, "Learn Hyprland");
    }

    #[test]
    fn parses_hand_written_markdown() {
        let raw = "\
# My Tasks

## Today
- [ ] Drink water
- [x] Wake up

## This Week
- [ ] Email dentist

## Someday
- [ ] Visit Hokkaido
";
        let parsed = parse(raw);
        assert_eq!(parsed.today.len(), 2);
        assert_eq!(parsed.today[0].text, "Drink water");
        assert!(parsed.today[1].done);
        assert_eq!(parsed.week[0].text, "Email dentist");
        assert_eq!(parsed.someday[0].text, "Visit Hokkaido");
    }

    #[test]
    fn ignores_non_task_lines() {
        let raw = "## Today\nrandom note\n- [ ] do thing\n";
        let parsed = parse(raw);
        assert_eq!(parsed.today.len(), 1);
        assert_eq!(parsed.today[0].text, "do thing");
    }
}
