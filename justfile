name := 'cosmic-applet-todo'
appid := 'dev.thomasverhappen.CosmicAppletTodo'

prefix := env_var_or_default('PREFIX', '/usr')
bindir := prefix / 'bin'
sharedir := prefix / 'share'
appdir := sharedir / 'applications'

# User install paths (used by `just install-user`)
user_prefix := env_var('HOME') / '.local'
user_bindir := user_prefix / 'bin'
user_appdir := user_prefix / 'share' / 'applications'
user_systemd := env_var('HOME') / '.config' / 'systemd' / 'user'

cargo_target := 'target' / 'release' / name

# Default recipe
default: build

# Build the release binary
build:
    cargo build --release

# Run cargo check
check:
    cargo check

# Run the unit tests
test:
    cargo test

# Format the code
fmt:
    cargo fmt

# Lint with clippy
lint:
    cargo clippy --all-targets -- -D warnings

# Run fmt-check, clippy, and tests (used in CI)
ci: check
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
    cargo test

# Clean build artifacts
clean:
    cargo clean

# Install system-wide (requires root). Usage: sudo just install
install: build
    install -Dm0755 {{cargo_target}} {{bindir / name}}
    install -Dm0755 data/todo-daily-checkin.sh {{bindir}}/todo-daily-checkin.sh
    install -Dm0644 data/{{appid}}.desktop {{appdir / appid}}.desktop

# Uninstall system-wide
uninstall:
    rm -f {{bindir / name}}
    rm -f {{bindir}}/todo-daily-checkin.sh
    rm -f {{appdir / appid}}.desktop

# Install for the current user only (no root, recommended)
install-user: build
    install -Dm0755 {{cargo_target}} {{user_bindir / name}}
    install -Dm0755 data/todo-daily-checkin.sh {{user_bindir}}/todo-daily-checkin.sh
    install -Dm0644 data/{{appid}}.desktop {{user_appdir / appid}}.desktop
    install -Dm0644 data/todo-daily-checkin.service {{user_systemd}}/todo-daily-checkin.service
    install -Dm0644 data/todo-daily-checkin.timer {{user_systemd}}/todo-daily-checkin.timer
    systemctl --user daemon-reload
    systemctl --user enable --now todo-daily-checkin.timer
    @echo
    @echo 'Installed. Add the applet via COSMIC Settings → Desktop → Panel → Add Applet → Todo.'

# Uninstall the user installation
uninstall-user:
    -systemctl --user disable --now todo-daily-checkin.timer
    rm -f {{user_bindir / name}}
    rm -f {{user_bindir}}/todo-daily-checkin.sh
    rm -f {{user_appdir / appid}}.desktop
    rm -f {{user_systemd}}/todo-daily-checkin.service
    rm -f {{user_systemd}}/todo-daily-checkin.timer
    systemctl --user daemon-reload
