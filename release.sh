#!/bin/bash

# release.sh will hopefully allow us to publish all of the necessary crates in
# this repo in the right order. It is assumed that only one person will be
# releasing all crates at the same time.
#
# It has a default set of crates it will publish, which can be overridden by
# way of command line arguments:
#
#   # Release all packages
#   ./release.sh
#
#   # Just release the proto and tendermint crates, but nothing else
#   ./release.sh tendermint-proto tendermint
#
# The script is idempotent: crate versions that have already been published
# are skipped, so it can simply be rerun if publishing fails partway through.

set -eo pipefail

# A space-separated list of all the crates we want to publish, in the order in
# which they must be published. It's important to respect this order, since
# each subsequent crate depends on one or more of the preceding ones.
DEFAULT_CRATES="tendermint-proto \
  tendermint-std-ext \
  tendermint \
  tendermint-config \
  tendermint-abci \
  tendermint-rpc \
  tendermint-p2p \
  tendermint-light-client-verifier \
  tendermint-light-client \
  tendermint-light-client-detector \
  tendermint-light-client-cli \
  tendermint-light-client-js \
  tendermint-testgen"

# Allows us to override the crates we want to publish.
CRATES=${*:-${DEFAULT_CRATES}}
CRATES_IO_USER_AGENT="${GITHUB_REPOSITORY} (${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY})"

# Additional flags to pass to the "cargo publish" operation for every crate we
# publish.
CARGO_PUBLISH_FLAGS=""

# Allow us to specify a crates.io API token via environment variables. Mostly
# for CI use.
if [ -n "${CRATES_TOKEN}" ]; then
  CARGO_PUBLISH_FLAGS="${CARGO_PUBLISH_FLAGS} --token ${CRATES_TOKEN}"
fi

get_manifest_path() {
  cargo metadata --format-version 1 | jq -r '.packages[]|select(.name == "'"${1}"'")|.manifest_path'
}

get_local_version() {
  cargo metadata --format-version 1 | jq -r '.packages[]|select(.name == "'"${1}"'")|.version'
}

check_version_online() {
  local response status
  response="$(curl -sS -A "${CRATES_IO_USER_AGENT}" -w '\n%{http_code}' "https://crates.io/api/v1/crates/${1}/${2}")"
  status="${response##*$'\n'}"
  case "${status}" in
    200) printf '%s' "${response%$'\n'*}" | jq -r '.version.created_at' ;;
    404) ;;
    *)
      echo "ERROR: crates.io returned HTTP ${status} when checking ${1} ${2}" >&2
      exit 1
      ;;
  esac
}

publish() {
  local log
  log="$(mktemp)"
  echo "Publishing crate $1..."
  if ! cargo publish --manifest-path "$(get_manifest_path "${1}")" ${CARGO_PUBLISH_FLAGS} 2>&1 | tee "${log}"; then
    # The crates.io API can lag behind the index, so check_version_online may
    # miss a version that has already been published (e.g. by a previous run).
    if grep -qE "already (exists|uploaded)" "${log}"; then
      echo "${1} has already been published, skipping"
    else
      rm -f "${log}"
      exit 1
    fi
  fi
  rm -f "${log}"
  echo ""
}

wait_until_available() {
  echo "Waiting for crate ${1} to become available via crates.io..."
  for retry in {1..5}; do
    sleep 5
    ONLINE_DATE="$(check_version_online "${1}" "${2}")"
    if [ -n "${ONLINE_DATE}" ]; then
      echo "Crate ${crate} is now available online"
      break
    else
      if [ "${retry}" == 5 ]; then
        echo "ERROR: Crate should have become available by now"
        exit 1
      else
        echo "Not available just yet. Waiting a few seconds..."
      fi
    fi
  done
  echo "Waiting an additional 10 seconds for crate to propagate through CDN..."
  sleep 10
}

echo "Attempting to publish crate(s): ${CRATES}"

for crate in ${CRATES}; do
  VERSION="$(get_local_version "${crate}")"
  if [ -z "${VERSION}" ]; then
    echo "ERROR: crate ${crate} not found in this workspace" >&2
    exit 1
  fi
  ONLINE_DATE="$(check_version_online "${crate}" "${VERSION}")"
  echo "${crate} version number: ${VERSION}"
  if [ -n "${ONLINE_DATE}" ]; then
    echo "${crate} ${VERSION} has already been published at ${ONLINE_DATE}, skipping"
    continue
  fi

  publish "${crate}"
  wait_until_available "${crate}" "${VERSION}"
done
