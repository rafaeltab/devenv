#!/bin/bash

# macOS ships with Bash 3.2, which lacks associative arrays.
# Use parallel arrays for shorthands (keys) and paths (values).

# Get the current working directory
PWD_PATH="$(pwd)"

# Expand HOME once
HOME_DIR="$HOME"

# Define shorthands (keys) and corresponding paths (values) in parallel arrays
shorthands=( "@raf-src" "@meth-src" "@src" "~" )
paths=( "$HOME_DIR/source/rafael" "$HOME_DIR/source/meth" "$HOME_DIR/home/source" "$HOME_DIR" )

# Replace the first occurrence of each path with its shorthand
REPLACED_PWD="$PWD_PATH"
for i in "${!shorthands[@]}"; do
  shorthand="${shorthands[$i]}"
  path="${paths[$i]}"

  # Use parameter expansion for simple, fast single replacement
  if [[ "$REPLACED_PWD" == *"$path"* ]]; then
    REPLACED_PWD="${REPLACED_PWD/$path/$shorthand}"
  fi
done

# Print the replaced path
echo "$REPLACED_PWD"
