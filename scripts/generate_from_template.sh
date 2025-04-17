#!/bin/bash
set -e

if [ -z "$1" ] || [ -z "$2" ]; then
  echo "Usage: $0 <template_file_path> <output_file_path> [env_file_path]"
  exit 1
fi

TEMPLATE_FILE="$1"
OUTPUT_FILE="$2"
# If [env_file_path] is not provided, it defaults to ./.env
ENV_FILE="${3:-./.env}"

if [ ! -f "$TEMPLATE_FILE" ]; then
  echo "Template file $TEMPLATE_FILE not found!"
  exit 1
fi

# Extract placeholders in the format <VAR> from the template file.
mapfile -t PLACEHOLDERS < <(grep -oP '<\K[^>]+(?=>)' "$TEMPLATE_FILE")

MISSING_VARS=()

# Check each placeholder for an environment variable, first from the current environment,
# then from the provided .env file if not set.
for PLACEHOLDER in "${PLACEHOLDERS[@]}"; do
  ENV_VAR="$PLACEHOLDER"
  ENV_VALUE="${!ENV_VAR}"

  if [ -z "$ENV_VALUE" ] && [ -f "$ENV_FILE" ]; then
    ENV_VALUE=$(grep "^${ENV_VAR}=" "$ENV_FILE" | head -n 1 | cut -d '=' -f2-)
  fi

  if [ -z "$ENV_VALUE" ]; then
    MISSING_VARS+=("$ENV_VAR")
  fi
done

if [ ${#MISSING_VARS[@]} -ne 0 ]; then
  echo "The following environment variables are not set: ${MISSING_VARS[*]}"
  exit 1
fi

cp "$TEMPLATE_FILE" "$OUTPUT_FILE"
FILE_PATH="$OUTPUT_FILE"

for PLACEHOLDER in "${PLACEHOLDERS[@]}"; do
  ENV_VAR="$PLACEHOLDER"
  ENV_VALUE="${!ENV_VAR}"

  if [ -z "$ENV_VALUE" ] && [ -f "$ENV_FILE" ]; then
    ENV_VALUE=$(grep "^${ENV_VAR}=" "$ENV_FILE" | head -n 1 | cut -d '=' -f2-)
  fi

  ESCAPED_VALUE=$(echo "$ENV_VALUE" | sed -e 's/[\/&]/\\&/g' -e 's/|/\\|/g' -e 's/`/\\`/g' -e 's/\$/\\\$/g')
  sed -i "s|<$PLACEHOLDER>|$ESCAPED_VALUE|g" "$FILE_PATH"
done

echo "Replacements complete. Modified file saved as $OUTPUT_FILE."
