# frozen_string_literal: true

require_relative "lib/ironcalc/version"

Gem::Specification.new do |spec|
  spec.name = "ironcalc"
  spec.version = IronCalc::VERSION
  spec.authors = ["Nicolás Hatcher"]
  spec.email = ["nicolas@theuniverse.today"]

  spec.summary = "Create, edit and evaluate Excel spreadsheets"
  spec.description = "Ruby bindings for IronCalc, a modern spreadsheet engine written in Rust. " \
                     "Read, write and evaluate xlsx workbooks, manage sheets, cells, styles and formulas."
  spec.homepage = "https://www.ironcalc.com/"
  spec.licenses = ["MIT", "Apache-2.0"]
  spec.required_ruby_version = ">= 3.0.0"
  # rb_sys / rubygems support for Cargo-based extensions.
  spec.required_rubygems_version = ">= 3.3.11"

  spec.metadata["homepage_uri"] = spec.homepage
  spec.metadata["source_code_uri"] = "https://github.com/ironcalc/IronCalc"
  spec.metadata["bug_tracker_uri"] = "https://github.com/ironcalc/IronCalc/issues"
  spec.metadata["changelog_uri"] = "https://github.com/ironcalc/IronCalc/releases"
  spec.metadata["rubygems_mfa_required"] = "true"

  spec.files = Dir[
    "lib/**/*.rb",
    "ext/**/*.{rs,rb,toml,lock}",
    "Cargo.toml",
    "Cargo.lock",
    ".cargo/config.toml",
    "README.md",
    "LICENSE-MIT.md",
    "LICENSE-Apache-2.0.md"
  ]
  spec.require_paths = ["lib"]
  spec.extensions = ["ext/ironcalc/extconf.rb"]

  # Needed at build time so `gem install` can compile the Rust extension.
  spec.add_dependency "rb_sys", "~> 0.9.91"

  spec.add_development_dependency "rake", "~> 13.0"
  spec.add_development_dependency "rake-compiler", "~> 1.2"
  spec.add_development_dependency "minitest", "~> 5.0"
end
