# Security Policy

## Supported versions

Only the latest release on `main` is supported. This is a small project — there is no LTS or backport policy.

## Reporting a vulnerability

Please **do not** open a public GitHub issue for security-related problems.

Email **business@thomasverhappen.dev** with:

- A description of the issue and its impact
- Steps to reproduce
- The affected version (commit SHA or tag)
- Any suggested mitigation, if you have one

You can expect:

- An acknowledgement within 7 days
- A status update within 14 days
- A patched release as soon as a fix is reasonably available

## Scope

In scope:

- Issues that allow arbitrary code execution via the applet
- Issues that read or write files outside `~/todo.md` and the configured config path
- Issues that crash the COSMIC panel from this applet
- Issues with the daily-check-in script that allow command injection through `~/todo.md` contents

Out of scope:

- Vulnerabilities in `libcosmic`, `cosmic-panel`, or other upstream dependencies — please report those to their respective projects
- Crashes caused by deliberately malformed `~/todo.md` files (the file is user-owned; treat the same as editing your own dotfiles)
- Social-engineering attacks against maintainers
