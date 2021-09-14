#!/bin/bash
# Copyright 2020-2021 The Datafuse Authors.
# SPDX-License-Identifier: Apache-2.0.

SCRIPT_PATH="$(cd "$(dirname "$0")" >/dev/null 2>&1 && pwd)"
cd "$SCRIPT_PATH/../.." || exit

echo "Build(DEBUG) start..."
cargo build --bin=databend-query --bin=databend-benchmark --bin=datafuse-store
echo "All done..."
