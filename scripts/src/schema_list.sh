file=$HOME/.rafaeltab/downloads/schema_catalog.json
selected=$(jq -r '.schemas[].name' "$file" | fzf --preview "jq --arg name {} '.schemas[] | select(.name == \$name)' $file")
jq --arg name "$selected" '.schemas[] | select(.name == $name)' "$file"
