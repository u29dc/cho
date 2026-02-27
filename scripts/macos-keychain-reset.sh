#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "Keychain reset helper is only available on Darwin."
  exit 0
fi

SERVICE_NAME="cho"
TOKENS_KEY="freeagent_tokens"

if security delete-generic-password -s "${SERVICE_NAME}" -a "${TOKENS_KEY}" >/dev/null 2>&1; then
  echo "Deleted keychain item ${SERVICE_NAME}/${TOKENS_KEY}."
else
  echo "No keychain item found for ${SERVICE_NAME}/${TOKENS_KEY}."
fi

echo "Run 'cho auth login --json' to recreate tokens and grant access once for the signed binaries."

