# IronCalc Ruby bindings

Ruby bindings for [IronCalc](https://www.ironcalc.com/), a modern spreadsheet
engine written in Rust. Create, read and manipulate xlsx files: manage sheets,
read and write cell values and styles, and — most importantly — evaluate
formulas.

The bindings are built with [magnus](https://github.com/matsadler/magnus) and
[rb-sys](https://github.com/oxidize-rb/rb-sys), and mirror the Python and Node
bindings so the API stays consistent across languages.

## Installation

```bash
gem install ironcalc
```

Or add it to your `Gemfile`:

```ruby
gem "ironcalc"
```

Ruby 3.0 or newer is required. A source install compiles the Rust extension, so
a Rust toolchain (`cargo`) must be available unless you install a precompiled
platform gem.

## Usage

There are two APIs:

* `IronCalc::Model` — the raw API. You drive evaluation yourself with
  `evaluate`.
* `IronCalc::UserModel` — the higher level API with an undo/redo aware diff
  queue (the same model the web app uses). It evaluates automatically.

```ruby
require "ironcalc"

# Arguments: name, locale, timezone, language id
model = IronCalc.create("Workbook1", "en", "UTC", "en")

# Cells are addressed as (sheet, row, column), all 1-based except the sheet
# index which is 0-based.
model.set_user_input(0, 1, 1, "2")
model.set_user_input(0, 1, 2, "3")
model.set_user_input(0, 1, 3, "=A1+B1")
model.evaluate

model.get_formatted_cell_value(0, 1, 3) # => "5"

model.save_to_xlsx("example.xlsx")
```

### The user model

```ruby
model = IronCalc.create_user_model("Workbook1", "en", "UTC", "en")
model.set_user_input(0, 1, 1, "=1+2")
model.get_formatted_cell_value(0, 1, 1) # => "3", no explicit evaluate needed

# Collaborative editing: ship the diff queue elsewhere and replay it.
diffs = model.flush_send_queue
other = IronCalc.create_user_model("Workbook1", "en", "UTC", "en")
other.apply_external_diffs(diffs)
```

### Styles

Styles are plain Ruby hashes with symbol keys. Read a cell's style, mutate it,
and write it back:

```ruby
style = model.get_cell_style(0, 1, 1)
# => {num_fmt: "general", fill: {}, font: {sz: 12, name: "Inter", ...}, ...}
style[:font][:b] = true            # bold
style[:fill][:color] = "#FFEB3B"   # yellow background
model.set_cell_style(0, 1, 1, style)
```

### Loading existing workbooks

```ruby
model = IronCalc.load_from_xlsx("example.xlsx", "en", "UTC", "en")

# Round-trip through the internal binary format.
bytes = model.to_bytes
restored = IronCalc.load_from_bytes(bytes, "en")
```

## Module functions

| Function | Returns |
| --- | --- |
| `IronCalc.create(name, locale, tz, language_id)` | `Model` |
| `IronCalc.load_from_xlsx(path, locale, tz, language_id)` | `Model` |
| `IronCalc.load_from_icalc(path, language_id)` | `Model` |
| `IronCalc.load_from_bytes(bytes, language_id)` | `Model` |
| `IronCalc.create_user_model(name, locale, tz, language_id)` | `UserModel` |
| `IronCalc.create_user_model_from_xlsx(path, locale, tz, language_id)` | `UserModel` |
| `IronCalc.create_user_model_from_icalc(path, language_id)` | `UserModel` |
| `IronCalc.create_user_model_from_bytes(bytes, language_id)` | `UserModel` |

Anything that can fail raises `IronCalc::Error`.

## Development

The crate lives in `ext/ironcalc` and is built via `rake-compiler`/`rb_sys`.

```bash
cd bindings/ruby
bundle install
bundle exec rake compile   # build the native extension
bundle exec rake test      # run the test suite
```

The system Ruby shipped with macOS is too old; use a brewed Ruby
(`brew install ruby`) or a version manager such as `rbenv`/`asdf` so that
`ruby --version` reports 3.0 or newer.

## License

Licensed under either of MIT or Apache-2.0 at your option.
