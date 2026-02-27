#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_DIR="${CHO_HOME:-${TOOLS_HOME:-$HOME/.tools}/cho}"

cd "${REPO_ROOT}"

cargo build --workspace --release

mkdir -p "${INSTALL_DIR}"

install_binary() {
  local binary_name="$1"
  local source_path="${REPO_ROOT}/target/release/${binary_name}"
  local target_path="${INSTALL_DIR}/${binary_name}"

  if [[ ! -f "${source_path}" ]]; then
    echo "Missing release binary: ${source_path}" >&2
    exit 1
  fi

  cp "${source_path}" "${target_path}"
  chmod 755 "${target_path}"
  echo "Installed ${binary_name} -> ${target_path}"
}

install_binary "cho"
install_binary "cho-tui"

if [[ "$(uname -s)" == "Darwin" ]]; then
  "${REPO_ROOT}/scripts/macos-codesign.sh" \
    "${INSTALL_DIR}/cho" \
    "${INSTALL_DIR}/cho-tui"
fi

