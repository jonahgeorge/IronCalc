use magnus::{Error, RHash, Ruby, Symbol};

use xlsx::base::types::{CellType, Color};

/// Converts an IronCalc [`CellType`] into a Ruby symbol such as `:number` or
/// `:logical_value`. Mirrors the discriminants exposed by the other bindings.
pub fn cell_type_to_symbol(ruby: &Ruby, cell_type: CellType) -> Symbol {
    let name = match cell_type {
        CellType::Number => "number",
        CellType::Text => "text",
        CellType::LogicalValue => "logical_value",
        CellType::ErrorValue => "error_value",
        CellType::Array => "array",
        CellType::CompoundData => "compound_data",
    };
    ruby.to_symbol(name)
}

/// Only RGB colors can be represented as a string. Theme colors and the absence
/// of a color are reported as `nil`, matching the Python and Node bindings.
pub fn color_to_option(color: Color) -> Option<String> {
    match color {
        Color::Rgb(s) => Some(s),
        Color::Theme(_, _) | Color::None => None,
    }
}

/// Builds the Ruby hash describing a single worksheet, with symbol keys
/// `:name`, `:state`, `:sheet_id` and `:color`.
pub fn worksheet_property_hash(
    ruby: &Ruby,
    name: String,
    state: String,
    sheet_id: u32,
    color: Color,
) -> Result<RHash, Error> {
    let hash = ruby.hash_new();
    hash.aset(ruby.to_symbol("name"), name)?;
    hash.aset(ruby.to_symbol("state"), state)?;
    hash.aset(ruby.to_symbol("sheet_id"), sheet_id)?;
    hash.aset(ruby.to_symbol("color"), color_to_option(color))?;
    Ok(hash)
}
