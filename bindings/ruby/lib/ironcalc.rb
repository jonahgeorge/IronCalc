# frozen_string_literal: true

require_relative "ironcalc/version"

# Load the compiled Rust extension. Precompiled ("fat") gems ship one binary per
# Ruby ABI under lib/ironcalc/<major.minor>/; source installs build it directly
# into lib/ironcalc/.
begin
  ruby_version = /(\d+\.\d+)/.match(RUBY_VERSION)
  require_relative "ironcalc/#{ruby_version}/ironcalc_ruby"
rescue LoadError
  require_relative "ironcalc/ironcalc_ruby"
end

# IronCalc is a modern spreadsheet engine. The Ruby bindings expose two entry
# points:
#
# * {IronCalc::Model} — the raw API; you call +evaluate+ yourself.
# * {IronCalc::UserModel} — the higher level API with an undo/redo aware diff
#   queue.
#
# Construct either through the module-level helpers, e.g. {IronCalc.create} or
# {IronCalc.create_user_model}.
module IronCalc
end
