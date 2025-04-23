#!/bin/bash

# Get the current working directory
PWD=$(pwd)

# Define the shorthands and their corresponding paths in an associative array
declare -A shorthands
shorthands["@raf-src"]="$HOME/source/rafael"
shorthands["@meth-src"]="$HOME/source/meth"
shorthands["@src"]="$HOME/home/source"
shorthands["~"]="$HOME"

# Iterate through the array and replace the paths with shorthands
REPLACED_PWD="$PWD"
for shorthand in "${!shorthands[@]}"; do
  path="${shorthands[$shorthand]}"
  REPLACED_PWD="${REPLACED_PWD/$path/$shorthand}"
done

# Print the replaced path
echo "$REPLACED_PWD"
