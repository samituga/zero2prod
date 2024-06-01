#!/bin/bash

if [ -z "$1" ]; then
  echo "Usage: $0 <file_path>"
  exit 1
fi

FILE_PATH="$1"

if [ ! -f "$FILE_PATH" ]; then
  echo "File $FILE_PATH not found!"
  exit 1
fi

PLACEHOLDERS=($(grep -oP '<\K[^>]+(?=>)' "$FILE_PATH"))

MISSING_VARS=()

for PLACEHOLDER in "${PLACEHOLDERS[@]}"; do
  ENV_VAR=$(echo $PLACEHOLDER)

  ENV_VALUE=$(eval echo \$$ENV_VAR)

  if [ -z "$ENV_VALUE" ]; then
    MISSING_VARS+=("$ENV_VAR")
  fi
done

if [ ${#MISSING_VARS[@]} -ne 0 ]; then
  echo "The following environment variables are not set: ${MISSING_VARS[*]}"
  exit 1
fi

for PLACEHOLDER in "${PLACEHOLDERS[@]}"; do
  ENV_VAR=$(echo $PLACEHOLDER)
  ENV_VALUE=$(eval echo \$$ENV_VAR)
  sed -i "s|<$PLACEHOLDER>|$ENV_VALUE|g" "$FILE_PATH"
done

BASENAME=$(basename "$FILE_PATH")

cp "$FILE_PATH" "./$BASENAME"

echo "Replacements complete. Modified file copied to the current directory as $BASENAME."
