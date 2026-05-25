# cosmic-applet-todo

A small, native [COSMIC](https://system76.com/cosmic) panel applet for daily and longer-horizon task planning. Lives in your top bar, backed by a plain `~/todo.md` file you can also edit by hand.

> Status: early. Works, but the libcosmic applet API is pre-1.0 and may shift.

## Why

COSMIC ships a full standalone Tasks app, but no panel-resident todo. This is for the case where you want a single keystroke from anywhere to a *short list of things*. Three fixed horizons keep planning honest:

- **Today** — things you actually expect to do today
- **This Week** — things on your radar
- **Someday** — things you don't want to forget

Click the panel icon → see/check/add tasks → click away. A daily desktop notification at 08:00 reminds you what's on Today.

## Install

### From source

Requires `cargo` (Rust ≥ 1.80), `libwayland-dev`, `libxkbcommon-dev`, and `just`.

```sh
git clone https://github.com/SenorBlub/cosmic-applet-todo
cd cosmic-applet-todo
just install-user
```

`just install-user` installs the binary, `.desktop` file, daily-check-in script, and a systemd user timer into `~/.local/` and `~/.config/`. No root required.

Then open **COSMIC Settings → Desktop → Panel → Add Applet** and pick **Todo**.

### AUR (Arch / CachyOS / EndeavourOS)

```sh
yay -S cosmic-applet-todo-git
```

*(Pending submission — see [`packaging/PKGBUILD`](packaging/PKGBUILD) in the meantime.)*

## File format

Tasks live in `~/todo.md` (configurable). It's plain GitHub-flavored markdown — edit in any editor and the applet picks up the changes next time you open the popup.

```markdown
<!-- Managed by cosmic-applet-todo. Sections: Today, This Week, Someday. -->

## Today

- [ ] Drink water
- [x] Wake up

## This Week

- [ ] Email dentist
- [ ] Finish the COSMIC applet

## Someday

- [ ] Visit Hokkaido
```

Rules:
- Section headings must be exactly `## Today`, `## This Week`, `## Someday`.
- Tasks must start with `- [ ]` or `- [x]`.
- Anything else in the file is ignored on parse (but will be removed on write — the applet rewrites the file when you mutate state from the popup).

## Configuration

Optional config at `~/.config/cosmic-applet-todo/config.toml`. Defaults shown:

```toml
# Path to the markdown file.
file_path = "~/todo.md"

# Tray icons. Any freedesktop icon name available in cosmic-icons.
icon_pending = "checkbox-symbolic"        # shown when Today has open tasks
icon_clear   = "checkbox-checked-symbolic"  # shown when Today is empty

# Popup window size in logical pixels.
popup_width  = 380
popup_height = 620
```

The popup uses COSMIC's spacing tokens for padding/gaps, so the layout follows your active COSMIC theme automatically.

### Daily check-in

A systemd user timer fires at 08:00 every day and sends a desktop notification summarizing Today.

Change the time:

```sh
systemctl --user edit todo-daily-checkin.timer
# Then under [Timer], override OnCalendar — e.g. OnCalendar=*-*-* 07:30:00
```

Disable entirely:

```sh
systemctl --user disable --now todo-daily-checkin.timer
```

Test now:

```sh
systemctl --user start todo-daily-checkin.service
```

## Development

```sh
just check      # fast type-check
just test       # unit tests (markdown parser)
just lint       # clippy with -D warnings
just fmt        # rustfmt
just ci         # everything CI runs
```

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for architecture notes.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
