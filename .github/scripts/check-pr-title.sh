#!/bin/bash
set -euo pipefail

echo "PR title: $TITLE"
if [ "$AUTHOR_TYPE" = "Bot" ]; then
  echo "Author is a bot, automations don't require linked issues"
  exit 0
fi

if echo "$TITLE" | grep -qE '^\[(#[1-9][0-9]*, )*#[1-9][0-9]*\]'; then
  echo "Check passed: PR title references one or more issues"
else
  echo "::error::PR title must start with one or more issue numbers"
  echo "::error::"
  echo "::error::correct examples:"
  echo "::error::[#123] Issue title"
  echo "::error::[#123, #567] Issue title"
  echo "::error::[#123] Issue title"
  echo "::error::"
  echo "::error::incorrect examples:"
  echo "::error::preceding text [#123] Issue title <-- title should start with the issues"
  echo "::error::#123, #567 Issue title <-- Missing braces ([...])"
  exit 1
fi
