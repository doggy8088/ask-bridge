#!/usr/bin/env bash

# Exit on error, undefined variables, and pipe failures
set -euo pipefail

VERSION="0.2.4"

show_version() {
  echo "$(basename "$0") version $VERSION"
  exit 0
}

show_help() {
  cat <<EOF
Usage:
  $(basename "$0") [-p PROVIDER] [USER_PROMPT]
  echo "USER_PROMPT" | $(basename "$0") [-p PROVIDER]

Options:
  -p    AI provider: 'chatgpt' or 'gemini' (default: 'chatgpt')
  -v    Show version information
  -h    Show this help message
EOF
  exit 0
}

usage_error() {
  cat <<EOF >&2
Usage:
  $(basename "$0") [-p PROVIDER] [USER_PROMPT]
  echo "USER_PROMPT" | $(basename "$0") [-p PROVIDER]

Options:
  -p    AI provider: 'chatgpt' or 'gemini' (default: 'chatgpt')
  -v    Show version information
  -h    Show this help message
EOF
  exit 1
}

PROMPT=""
PROVIDER="chatgpt"

# Parse options
while getopts "p:hv" opt; do
  case "$opt" in
    p)
      PROVIDER=$(echo "$OPTARG" | tr '[:upper:]' '[:lower:]')
      ;;
    v)
      show_version
      ;;
    h)
      show_help
      ;;
    *)
      usage_error
      ;;
  esac
done

# Shift off the options parsed by getopts
shift $((OPTIND-1))

# If -i was not used but positional arguments are provided, use them as the prompt
if [ -z "$PROMPT" ] && [ $# -gt 0 ]; then
  PROMPT="$*"
fi

# Validate provider
if [ "$PROVIDER" != "chatgpt" ] && [ "$PROVIDER" != "gemini" ]; then
  echo "Error: Invalid provider '$PROVIDER'." >&2
  usage_error
fi

# Determine base URL based on provider
if [ "$PROVIDER" = "chatgpt" ]; then
  BASE_URL="https://chatgpt.com/"
else
  BASE_URL="https://gemini.google.com/app"
fi

# If PROMPT is empty and stdin is not a TTY, read from stdin (pipe)
if [ -z "$PROMPT" ] && [ ! -t 0 ]; then
  PROMPT=$(cat)
fi

# If PROMPT is still empty, show help message by default
if [ -z "$PROMPT" ]; then
  show_help
fi

# Conditionally encode the prompt based on length
if [ "${#PROMPT}" -gt 64 ]; then
  # Use base64 encoding for prompts longer than 64 characters
  B64_PROMPT=$(printf '%s' "$PROMPT" | base64 | tr -d '\n\r')
  URL="${BASE_URL}#autoSubmit=true&prompt=${B64_PROMPT}"
else
  # Use URL encoding for prompts 64 characters or fewer
  ENCODED_PROMPT=$(printf '%s' "$PROMPT" | python3 -c "import urllib.parse, sys; print(urllib.parse.quote(sys.stdin.read(), safe=''))")
  URL="${BASE_URL}#autoSubmit=true&prompt=${ENCODED_PROMPT}"
fi

# Open the URL in the default browser
echo "Opening ${PROVIDER} with your prompt..."

if [[ "$OSTYPE" == "darwin"* ]]; then
  open "$URL"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
  if command -v xdg-open >/dev/null 2>&1; then
    xdg-open "$URL"
  else
    echo "Linux detected, but xdg-open command not found. Here is the URL:"
    echo "$URL"
  fi
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
  start "$URL"
else
  # Generic fallbacks
  if command -v open >/dev/null 2>&1; then
    open "$URL"
  elif command -v xdg-open >/dev/null 2>&1; then
    xdg-open "$URL"
  else
    echo "Could not detect browser-opening tool. Here is the URL:"
    echo "$URL"
  fi
fi
