#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "macOS code-signing status is only available on Darwin."
  exit 0
fi

INSTALL_DIR="${CHO_HOME:-${TOOLS_HOME:-$HOME/.tools}/cho}"
SERVICE_NAME="cho"
TOKENS_KEY="freeagent_tokens"

extract_codesign_field() {
  local binary_path="$1"
  local field_name="$2"
  local output

  output="$(codesign --display --verbose=2 "${binary_path}" 2>&1 || true)"
  printf "%s\n" "${output}" | awk -F= -v field="${field_name}" '$1 == field { print $2; exit }'
}

echo "Install directory: ${INSTALL_DIR}"
echo

echo "Available code-signing identities:"
security find-identity -v -p codesigning || true
echo

for binary_name in "cho" "cho-tui"; do
  binary_path="${INSTALL_DIR}/${binary_name}"
  echo "Binary: ${binary_path}"

  if [[ ! -f "${binary_path}" ]]; then
    echo "  missing"
    echo
    continue
  fi

  if codesign --verify --verbose=2 "${binary_path}" >/dev/null 2>&1; then
    identifier="$(extract_codesign_field "${binary_path}" "Identifier")"
    authority="$(extract_codesign_field "${binary_path}" "Authority")"
    echo "  signed: yes"
    echo "  identifier: ${identifier:-unknown}"
    echo "  authority: ${authority:-unknown}"
  else
    echo "  signed: no (or invalid signature)"
  fi
  echo
done

echo "Keychain item (${SERVICE_NAME}/${TOKENS_KEY}):"
if security find-generic-password -s "${SERVICE_NAME}" -a "${TOKENS_KEY}" >/dev/null 2>&1; then
  echo "  present"
else
  echo "  not present"
fi
