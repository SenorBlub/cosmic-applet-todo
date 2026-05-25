<!--
PR titles must reference an issue. Accepted formats:

  #42 Short description
  #42 - Short description
  feat(#42): Short description

Or, if the title doesn't lead with the number, the body must contain one of:

  closes #42
  fixes #42
  resolves #42
  refs #42

The PR / issue-link check enforces this on every PR.
-->

# Summary

<!-- One or two sentences on what this PR does and why. -->

## Changes

<!-- Bullet list of the substantive changes. -->

-

## Linked issue

<!-- Required. Pick the closest verb. -->

Closes #

## Checklist

- [ ] `just ci` passes locally (fmt, check, clippy with `-D warnings`, tests)
- [ ] New config keys (if any) have sensible defaults that preserve current behavior
- [ ] No new hardcoded spacing or color values — used `cosmic::theme::active().cosmic().spacing` tokens
- [ ] README updated if user-facing behavior changed
