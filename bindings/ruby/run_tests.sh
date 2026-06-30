#!/bin/bash
set -euo pipefail

# Builds the native extension and runs the test suite. Requires Ruby >= 3.0
# (the system Ruby on macOS is too old; use a version manager or a brewed Ruby).

cd "$(dirname "$0")"

bundle install
bundle exec rake compile
bundle exec rake test
