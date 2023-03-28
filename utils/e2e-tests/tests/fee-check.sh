#!/bin/bash
set -euo pipefail

# Set up command.
COMMAND="$1"

# Make temporary test directory.
TEMPDIR="$(mktemp -d)"
trap 'rm -rf "$TEMPDIR"; pkill -P "$$"' EXIT

# Run the node.
"$COMMAND" --dev --base-path "$TEMPDIR" &

# Encoded balance transfer keep alive extrinsic.
#
# From:   Alice.
# To:     Bob.
# Amount: 10 HMND.
TRANSFER_EXTRINSIC="0x4d028400d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d01e4708233954d8201f6aae06760a5a943abe7453138cba7deebd215da3188ff19d56095d976773669f65c0eb8299f51ae107e10a2dbb63f8c530c8484b7e3238fb50000000803008eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48130000e8890423c78a"

# Obtain the extrinsic payment info data.
PAYMENT_INFO_JSON="$(yarn polkadot-js-api --ws "ws://127.0.0.1:9944" rpc.payment.queryInfo "$TRANSFER_EXTRINSIC")"

# Obtain the partial fee itself.
PARTIAL_FEE="$(echo "$PAYMENT_INFO_JSON" | grep "partialFee" | awk -v FS="(partialFee\": \"| mUnit)" '{print $2}')"

printf "Test succeded\n" >&2
