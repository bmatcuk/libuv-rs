#!/bin/bash
set -Exeuo pipefail

print_status() {
  local level="$1"
  local body="${2//%/%25}"
  body="${body//$'\r'/}"
  body="${body//$'\n'/%0A}"

  echo "::$level::$body"
}

# setup git
git config --local user.name $GIT_USER_NAME
git config --local user.email $GIT_USER_EMAIL

# make sure to fetch tags
git fetch --tags

# get version info
NEW_TAG=$(grep '^version =' Cargo.toml | awk -F'"' '{print "v" $2}')
PREV_TAG=$({ echo $NEW_TAG; git tag; } | sort -V | grep -B1 $NEW_TAG | head -n 1)

# set tags
git tag -a $NEW_TAG -m "${NEW_TAG}"
git push --tags

# build RELEASELOG.md and release
[ "$PREV_TAG" != "$NEW_TAG" ] && { echo -e "## Changelog\n\n"; git log --pretty=format:"%h %s" "${PREV_TAG}.." 2>/dev/null; } > RELEASELOG.md || echo "" > RELEASELOG.md
gh release create "$NEW_TAG" README.md --notes-file RELEASELOG.md --title "$NEW_TAG"

MSG="libuv-rs $NEW_TAG published"
print_status notice "$MSG"
