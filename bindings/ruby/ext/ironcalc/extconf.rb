require "mkmf"
require "rb_sys/mkmf"

# Builds the Rust crate in this directory and installs the resulting library as
# `lib/ironcalc/ironcalc_ruby.<dlext>`, which `lib/ironcalc.rb` then requires.
create_rust_makefile("ironcalc/ironcalc_ruby")
