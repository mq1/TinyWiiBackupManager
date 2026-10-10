#!/bin/bash

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
grep -m1 '^version = ' "${SCRIPT_DIR}/../Cargo.toml" | cut -d'"' -f2 | tr -d '\n'

