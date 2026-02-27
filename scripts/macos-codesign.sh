#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  exit 0
fi

if [[ "${CHO_CODESIGN_DISABLE:-0}" == "1" ]]; then
  echo "Skipping macOS code signing (CHO_CODESIGN_DISABLE=1)"
  exit 0
fi

if [[ "$#" -eq 0 ]]; then
  echo "Usage: $0 <binary-path> [binary-path...]" >&2
  exit 1
fi

discover_codesign_identity() {
  security find-identity -v -p codesigning 2>/dev/null \
    | awk -F'"' '/"[^"]+"/ { print $2; exit }'
}

extract_codesign_field() {
  local binary_path="$1"
  local field_name="$2"
  local output

  output="$(codesign --display --verbose=2 "${binary_path}" 2>&1 || true)"
  printf "%s\n" "${output}" | awk -F= -v field="${field_name}" '$1 == field { print $2; exit }'
}

infer_identifier() {
  local binary_name="$1"
  local prefix="${CHO_CODESIGN_IDENTIFIER_PREFIX:-com.cho}"

  case "${binary_name}" in
    cho)
      printf "%s.cli" "${prefix}"
      ;;
    cho-tui)
      printf "%s.tui" "${prefix}"
      ;;
    *)
      printf "%s.%s" "${prefix}" "${binary_name}"
      ;;
  esac
}

IDENTITY="${CHO_CODESIGN_IDENTITY:-}"
if [[ -z "${IDENTITY}" ]]; then
  IDENTITY="$(discover_codesign_identity || true)"
fi

if [[ -z "${IDENTITY}" ]]; then
  message="No code-signing identity found. Set CHO_CODESIGN_IDENTITY or install a local Code Signing identity in Keychain Access."
  if [[ "${CHO_CODESIGN_REQUIRED:-0}" == "1" ]]; then
    echo "${message}" >&2
    exit 1
  fi
  echo "Warning: ${message}" >&2
  exit 0
fi

for binary_path in "$@"; do
  if [[ ! -f "${binary_path}" ]]; then
    echo "Missing binary to sign: ${binary_path}" >&2
    exit 1
  fi

  binary_name="$(basename "${binary_path}")"
  identifier="$(infer_identifier "${binary_name}")"

  codesign \
    --force \
    --sign "${IDENTITY}" \
    --identifier "${identifier}" \
    --timestamp=none \
    "${binary_path}"

  codesign --verify --verbose=2 "${binary_path}" >/dev/null

  authority="$(extract_codesign_field "${binary_path}" "Authority")"
  echo "Signed ${binary_path} (${identifier}) with ${authority:-${IDENTITY}}"
done
