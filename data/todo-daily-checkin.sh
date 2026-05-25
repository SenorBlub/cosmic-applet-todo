#!/usr/bin/env bash
set -eu

TODO_FILE="${HOME}/todo.md"

if [ ! -f "$TODO_FILE" ]; then
    notify-send -a "Todo" "Daily check-in" "No ~/todo.md yet — open the panel applet to start."
    exit 0
fi

# Extract lines under "## Today" up to the next "## " heading, then filter unchecked "- [ ]" items.
TODAY=$(awk '
    /^## Today[[:space:]]*$/ { in_section=1; next }
    /^## / { in_section=0 }
    in_section && /^- \[ \]/ {
        sub(/^- \[ \] */, "")
        print "• " $0
    }
' "$TODO_FILE")

COUNT=$(printf '%s\n' "$TODAY" | grep -c '^•' || true)

if [ "$COUNT" -eq 0 ]; then
    notify-send -a "Todo" "Daily check-in" "Today's list is clear."
else
    notify-send -a "Todo" -u normal -t 15000 \
        "Today — $COUNT task(s)" \
        "$TODAY"
fi
