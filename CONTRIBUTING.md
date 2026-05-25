# Contributing

Thanks for considering a contribution. This project is small and tries to stay that way.

## Quickstart

```sh
git clone https://github.com/SenorBlub/cosmic-applet-todo
cd cosmic-applet-todo
just build      # release build
just install-user
```

## Development loop

```sh
just check      # fast type check
just test       # unit tests for the markdown parser
just lint       # clippy
just fmt        # rustfmt
just ci         # everything CI runs
```

Re-install after changes:

```sh
just install-user
# Then in COSMIC: right-click the applet → Remove, then Add Applet → Todo
# (or just reboot — the panel re-reads applets on start)
```

## Architecture

Three modules:

- `src/config.rs` — loads `~/.config/cosmic-applet-todo/config.toml`. All knobs live here.
- `src/storage.rs` — markdown parser/writer for `~/todo.md`. Atomic write via temp+rename. Pure functions, fully tested.
- `src/window.rs` — the libcosmic applet itself. State machine + view code.

The applet is a `cosmic::Application` with `Flags = Config`. The popup is constructed via `cosmic::surface::action::app_popup`. All spacing uses `cosmic::theme::active().cosmic().spacing` tokens — please don't hardcode pixel values.

## Pull requests

- Run `just ci` locally before pushing.
- Keep changes focused — one feature or fix per PR.
- New features that add config knobs: add a default that mirrors current behavior.
- New widgets: use libcosmic widgets and cosmic spacing tokens.
- Don't add dependencies without a justification in the PR description.

## Scope

In scope:
- Better keyboard/accessibility support
- Additional horizons (configurable)
- Richer markdown (tags, due dates) — only if it stays compatible with hand-edited files
- Better empty/error states
- Native COSMIC config integration (cosmic-config) replacing the TOML file

Out of scope:
- Cloud sync / accounts
- Notifications beyond the daily check-in
- Replacing the markdown backend with something binary

## License

By contributing you agree your contribution is dual-licensed under MIT and Apache-2.0, matching the project.
