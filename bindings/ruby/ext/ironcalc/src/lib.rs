//! Ruby bindings for the IronCalc spreadsheet engine.
//!
//! This crate exposes two classes through [magnus](https://docs.rs/magnus):
//!
//! * `IronCalc::Model` — the low level ("raw") API where you drive evaluation
//!   yourself.
//! * `IronCalc::UserModel` — the higher level API with undo/redo and a diff
//!   queue, mirroring what the web application uses.
//!
//! The surface intentionally tracks the Python bindings so the three language
//! bindings stay in lock-step.

use std::cell::RefCell;

use magnus::{
    function, method, prelude::*, value::Lazy, Error, ExceptionClass, RArray, RString, Ruby, Value,
};

use xlsx::base::expressions::types::Area;
use xlsx::base::types::{Color, Style, Workbook};
use xlsx::base::{Model as XModel, UserModel as XUserModel};
use xlsx::export::{save_to_icalc, save_to_xlsx};
use xlsx::import;

mod types;
use types::{cell_type_to_symbol, worksheet_property_hash};

/// `IronCalc::Error`, the exception every fallible binding raises on failure.
static ERROR: Lazy<ExceptionClass> = Lazy::new(|ruby| {
    let module = ruby
        .define_module("IronCalc")
        .expect("could not define the IronCalc module");
    module
        .define_error("Error", ruby.exception_standard_error())
        .expect("could not define IronCalc::Error")
});

/// Turns any displayable error (most of the engine returns `String` errors)
/// into an `IronCalc::Error` Ruby exception.
fn err<E: std::fmt::Display>(e: E) -> Error {
    let ruby = Ruby::get().expect("IronCalc called outside of a Ruby thread");
    Error::new(ruby.get_inner(&ERROR), e.to_string())
}

/// The engine keeps `&'static str` locale/language references, so the bindings
/// leak the incoming strings exactly like the Python bindings do. These live
/// for the duration of the process, which is the expected lifetime for a
/// workbook's locale.
fn leak_str(s: &str) -> &'static str {
    Box::leak(s.to_owned().into_boxed_str())
}

fn rstring_to_vec(bytes: &RString) -> Vec<u8> {
    // Safety: we copy the bytes out immediately and never hold the slice across
    // a call back into Ruby, so the underlying buffer cannot move or be freed.
    unsafe { bytes.as_slice().to_vec() }
}

// ---------------------------------------------------------------------------
// UserModel — the higher level API
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "IronCalc::UserModel", free_immediately, size)]
struct UserModel(RefCell<XUserModel<'static>>);

impl UserModel {
    fn save_to_xlsx(&self, file: String) -> Result<(), Error> {
        let model = self.0.borrow();
        save_to_xlsx(model.get_model(), &file).map_err(err)
    }

    fn save_to_icalc(&self, file: String) -> Result<(), Error> {
        let model = self.0.borrow();
        save_to_icalc(model.get_model(), &file).map_err(err)
    }

    fn apply_external_diffs(&self, external_diffs: RString) -> Result<(), Error> {
        self.0
            .borrow_mut()
            .apply_external_diffs(&rstring_to_vec(&external_diffs))
            .map_err(err)
    }

    fn flush_send_queue(&self) -> RString {
        let ruby = Ruby::get().expect("IronCalc called outside of a Ruby thread");
        ruby.str_from_slice(&self.0.borrow_mut().flush_send_queue())
    }

    fn set_user_input(&self, sheet: u32, row: i32, column: i32, value: String) -> Result<(), Error> {
        self.0
            .borrow_mut()
            .set_user_input(sheet, row, column, &value)
            .map_err(err)
    }

    fn get_formatted_cell_value(&self, sheet: u32, row: i32, column: i32) -> Result<String, Error> {
        self.0
            .borrow()
            .get_formatted_cell_value(sheet, row, column)
            .map_err(err)
    }

    /// Bounds of all non-empty cells as `[min_row, max_row, min_column, max_column]`.
    /// An empty sheet reports `[1, 1, 1, 1]`.
    fn get_sheet_dimensions(&self, sheet: u32) -> Result<(i32, i32, i32, i32), Error> {
        let model = self.0.borrow();
        let worksheet = model.get_model().workbook.worksheet(sheet).map_err(err)?;
        let dimension = worksheet.dimension();
        Ok((
            dimension.min_row,
            dimension.max_row,
            dimension.min_column,
            dimension.max_column,
        ))
    }

    fn to_bytes(&self) -> RString {
        let ruby = Ruby::get().expect("IronCalc called outside of a Ruby thread");
        ruby.str_from_slice(&self.0.borrow().to_bytes())
    }
}

// ---------------------------------------------------------------------------
// Model — the raw API
// ---------------------------------------------------------------------------

#[magnus::wrap(class = "IronCalc::Model", free_immediately, size)]
struct Model(RefCell<XModel<'static>>);

impl Model {
    fn save_to_xlsx(&self, file: String) -> Result<(), Error> {
        save_to_xlsx(&self.0.borrow(), &file).map_err(err)
    }

    fn save_to_icalc(&self, file: String) -> Result<(), Error> {
        save_to_icalc(&self.0.borrow(), &file).map_err(err)
    }

    fn to_bytes(&self) -> RString {
        let ruby = Ruby::get().expect("IronCalc called outside of a Ruby thread");
        ruby.str_from_slice(&self.0.borrow().to_bytes())
    }

    fn evaluate(&self) {
        self.0.borrow_mut().evaluate();
    }

    fn set_user_input(&self, sheet: u32, row: i32, column: i32, value: String) -> Result<(), Error> {
        self.0
            .borrow_mut()
            .set_user_input(sheet, row, column, value)
            .map_err(err)
    }

    fn clear_cell_contents(&self, sheet: u32, row: i32, column: i32) -> Result<(), Error> {
        let area = Area {
            sheet,
            row,
            column,
            width: 1,
            height: 1,
        };
        self.0.borrow_mut().range_clear_contents(&area).map_err(err)
    }

    fn get_cell_content(&self, sheet: u32, row: i32, column: i32) -> Result<String, Error> {
        self.0
            .borrow()
            .get_localized_cell_content(sheet, row, column)
            .map_err(err)
    }

    /// Returns the cell type as a symbol: `:number`, `:text`, `:logical_value`,
    /// `:error_value`, `:array` or `:compound_data`.
    fn get_cell_type(&self, sheet: u32, row: i32, column: i32) -> Result<magnus::Symbol, Error> {
        let ruby = Ruby::get().map_err(err)?;
        let cell_type = self
            .0
            .borrow()
            .get_cell_type(sheet, row, column)
            .map_err(err)?;
        Ok(cell_type_to_symbol(&ruby, cell_type))
    }

    fn get_formatted_cell_value(&self, sheet: u32, row: i32, column: i32) -> Result<String, Error> {
        self.0
            .borrow()
            .get_formatted_cell_value(sheet, row, column)
            .map_err(err)
    }

    /// Reads the full style of a cell as a Ruby hash. Pair it with
    /// `set_cell_style` to round-trip styling without hand-assembling the hash.
    fn get_cell_style(&self, sheet: u32, row: i32, column: i32) -> Result<Value, Error> {
        let ruby = Ruby::get().map_err(err)?;
        let style = self
            .0
            .borrow()
            .get_style_for_cell(sheet, row, column)
            .map_err(err)?;
        serde_magnus::serialize(&ruby, &style)
    }

    /// Applies a style hash (typically obtained from `get_cell_style`) to a cell.
    fn set_cell_style(
        &self,
        sheet: u32,
        row: i32,
        column: i32,
        style: Value,
    ) -> Result<(), Error> {
        let ruby = Ruby::get().map_err(err)?;
        let style: Style = serde_magnus::deserialize(&ruby, style)?;
        self.0
            .borrow_mut()
            .set_cell_style(sheet, row, column, &style)
            .map_err(err)
    }

    fn insert_rows(&self, sheet: u32, row: i32, row_count: i32) -> Result<(), Error> {
        self.0.borrow_mut().insert_rows(sheet, row, row_count).map_err(err)
    }

    fn insert_columns(&self, sheet: u32, column: i32, column_count: i32) -> Result<(), Error> {
        self.0
            .borrow_mut()
            .insert_columns(sheet, column, column_count)
            .map_err(err)
    }

    fn delete_rows(&self, sheet: u32, row: i32, row_count: i32) -> Result<(), Error> {
        self.0.borrow_mut().delete_rows(sheet, row, row_count).map_err(err)
    }

    fn delete_columns(&self, sheet: u32, column: i32, column_count: i32) -> Result<(), Error> {
        self.0
            .borrow_mut()
            .delete_columns(sheet, column, column_count)
            .map_err(err)
    }

    fn get_column_width(&self, sheet: u32, column: i32) -> Result<f64, Error> {
        self.0.borrow().get_column_width(sheet, column).map_err(err)
    }

    fn get_row_height(&self, sheet: u32, row: i32) -> Result<f64, Error> {
        self.0.borrow().get_row_height(sheet, row).map_err(err)
    }

    fn set_column_width(&self, sheet: u32, column: i32, width: f64) -> Result<(), Error> {
        self.0
            .borrow_mut()
            .set_column_width(sheet, column, width)
            .map_err(err)
    }

    fn set_row_height(&self, sheet: u32, row: i32, height: f64) -> Result<(), Error> {
        self.0.borrow_mut().set_row_height(sheet, row, height).map_err(err)
    }

    fn get_frozen_columns_count(&self, sheet: u32) -> Result<i32, Error> {
        self.0.borrow().get_frozen_columns_count(sheet).map_err(err)
    }

    fn get_frozen_rows_count(&self, sheet: u32) -> Result<i32, Error> {
        self.0.borrow().get_frozen_rows_count(sheet).map_err(err)
    }

    fn set_frozen_columns_count(&self, sheet: u32, column_count: i32) -> Result<(), Error> {
        self.0.borrow_mut().set_frozen_columns(sheet, column_count).map_err(err)
    }

    fn set_frozen_rows_count(&self, sheet: u32, row_count: i32) -> Result<(), Error> {
        self.0.borrow_mut().set_frozen_rows(sheet, row_count).map_err(err)
    }

    /// Returns an array of worksheet hashes (`:name`, `:state`, `:sheet_id`,
    /// `:color`).
    fn get_worksheets_properties(&self) -> Result<RArray, Error> {
        let ruby = Ruby::get().map_err(err)?;
        let array = ruby.ary_new();
        for properties in self.0.borrow().get_worksheets_properties() {
            let hash = worksheet_property_hash(
                &ruby,
                properties.name,
                properties.state,
                properties.sheet_id,
                properties.color,
            )?;
            array.push(hash)?;
        }
        Ok(array)
    }

    fn set_sheet_color(&self, sheet: u32, color: String) -> Result<(), Error> {
        let color = Color::from_rgb(&color).map_err(err)?;
        self.0.borrow_mut().set_sheet_color(sheet, &color).map_err(err)
    }

    fn add_sheet(&self, sheet_name: String) -> Result<(), Error> {
        self.0.borrow_mut().add_sheet(&sheet_name).map_err(err)
    }

    fn new_sheet(&self) {
        self.0.borrow_mut().new_sheet();
    }

    fn delete_sheet(&self, sheet: u32) -> Result<(), Error> {
        self.0.borrow_mut().delete_sheet(sheet).map_err(err)
    }

    fn rename_sheet(&self, sheet: u32, new_name: String) -> Result<(), Error> {
        self.0
            .borrow_mut()
            .rename_sheet_by_index(sheet, &new_name)
            .map_err(err)
    }

    /// Bounds of all non-empty cells as `[min_row, max_row, min_column, max_column]`.
    /// An empty sheet reports `[1, 1, 1, 1]`.
    fn get_sheet_dimensions(&self, sheet: u32) -> Result<(i32, i32, i32, i32), Error> {
        let model = self.0.borrow();
        let worksheet = model.workbook.worksheet(sheet).map_err(err)?;
        let dimension = worksheet.dimension();
        Ok((
            dimension.min_row,
            dimension.max_row,
            dimension.min_column,
            dimension.max_column,
        ))
    }
}

// ---------------------------------------------------------------------------
// Module level constructors
// ---------------------------------------------------------------------------

/// Creates an empty workbook using the raw API.
fn create(name: String, locale: String, tz: String, language_id: String) -> Result<Model, Error> {
    let model = XModel::new_empty(
        leak_str(&name),
        leak_str(&locale),
        leak_str(&tz),
        leak_str(&language_id),
    )
    .map_err(err)?;
    Ok(Model(RefCell::new(model)))
}

/// Loads a workbook from an xlsx file using the raw API.
fn load_from_xlsx(
    file_path: String,
    locale: String,
    tz: String,
    language_id: String,
) -> Result<Model, Error> {
    let model = import::load_from_xlsx(&file_path, &locale, &tz, leak_str(&language_id)).map_err(err)?;
    Ok(Model(RefCell::new(model)))
}

/// Loads a workbook from the internal binary `.ic` representation.
fn load_from_icalc(file_name: String, language_id: String) -> Result<Model, Error> {
    let model = import::load_from_icalc(&file_name, leak_str(&language_id)).map_err(err)?;
    Ok(Model(RefCell::new(model)))
}

/// Loads a workbook from bytes in the internal binary `.ic` format (the same
/// format produced by `to_bytes` / `save_to_icalc`).
fn load_from_bytes(bytes: RString, language_id: String) -> Result<Model, Error> {
    let workbook: Workbook = bitcode::decode(&rstring_to_vec(&bytes)).map_err(err)?;
    let model = XModel::from_workbook(workbook, leak_str(&language_id)).map_err(err)?;
    Ok(Model(RefCell::new(model)))
}

/// Creates an empty workbook using the user-model API.
fn create_user_model(
    name: String,
    locale: String,
    tz: String,
    language_id: String,
) -> Result<UserModel, Error> {
    let model = XUserModel::new_empty(
        leak_str(&name),
        leak_str(&locale),
        leak_str(&tz),
        leak_str(&language_id),
    )
    .map_err(err)?;
    Ok(UserModel(RefCell::new(model)))
}

/// Creates a user model from an xlsx file.
fn create_user_model_from_xlsx(
    file_path: String,
    locale: String,
    tz: String,
    language_id: String,
) -> Result<UserModel, Error> {
    let model = import::load_from_xlsx(&file_path, &locale, &tz, leak_str(&language_id)).map_err(err)?;
    Ok(UserModel(RefCell::new(XUserModel::from_model(model))))
}

/// Creates a user model from the internal binary `.ic` representation.
fn create_user_model_from_icalc(file_name: String, language_id: String) -> Result<UserModel, Error> {
    let model = import::load_from_icalc(&file_name, leak_str(&language_id)).map_err(err)?;
    Ok(UserModel(RefCell::new(XUserModel::from_model(model))))
}

/// Creates a user model from bytes in the internal binary `.ic` format.
fn create_user_model_from_bytes(bytes: RString, language_id: String) -> Result<UserModel, Error> {
    let workbook: Workbook = bitcode::decode(&rstring_to_vec(&bytes)).map_err(err)?;
    let model = XModel::from_workbook(workbook, leak_str(&language_id)).map_err(err)?;
    Ok(UserModel(RefCell::new(XUserModel::from_model(model))))
}

// ---------------------------------------------------------------------------
// Module initialisation
// ---------------------------------------------------------------------------

#[magnus::init(name = "ironcalc_ruby")]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("IronCalc")?;

    // Force the error class to be defined so `rescue IronCalc::Error` works
    // even before any binding has raised. `IronCalc::VERSION` is owned by the
    // Ruby side (lib/ironcalc/version.rb).
    let _ = ruby.get_inner(&ERROR);

    module.define_module_function("create", function!(create, 4))?;
    module.define_module_function("load_from_xlsx", function!(load_from_xlsx, 4))?;
    module.define_module_function("load_from_icalc", function!(load_from_icalc, 2))?;
    module.define_module_function("load_from_bytes", function!(load_from_bytes, 2))?;
    module.define_module_function("create_user_model", function!(create_user_model, 4))?;
    module.define_module_function(
        "create_user_model_from_xlsx",
        function!(create_user_model_from_xlsx, 4),
    )?;
    module.define_module_function(
        "create_user_model_from_icalc",
        function!(create_user_model_from_icalc, 2),
    )?;
    module.define_module_function(
        "create_user_model_from_bytes",
        function!(create_user_model_from_bytes, 2),
    )?;

    let model = module.define_class("Model", ruby.class_object())?;
    model.define_method("save_to_xlsx", method!(Model::save_to_xlsx, 1))?;
    model.define_method("save_to_icalc", method!(Model::save_to_icalc, 1))?;
    model.define_method("to_bytes", method!(Model::to_bytes, 0))?;
    model.define_method("evaluate", method!(Model::evaluate, 0))?;
    model.define_method("set_user_input", method!(Model::set_user_input, 4))?;
    model.define_method("clear_cell_contents", method!(Model::clear_cell_contents, 3))?;
    model.define_method("get_cell_content", method!(Model::get_cell_content, 3))?;
    model.define_method("get_cell_type", method!(Model::get_cell_type, 3))?;
    model.define_method(
        "get_formatted_cell_value",
        method!(Model::get_formatted_cell_value, 3),
    )?;
    model.define_method("get_cell_style", method!(Model::get_cell_style, 3))?;
    model.define_method("set_cell_style", method!(Model::set_cell_style, 4))?;
    model.define_method("insert_rows", method!(Model::insert_rows, 3))?;
    model.define_method("insert_columns", method!(Model::insert_columns, 3))?;
    model.define_method("delete_rows", method!(Model::delete_rows, 3))?;
    model.define_method("delete_columns", method!(Model::delete_columns, 3))?;
    model.define_method("get_column_width", method!(Model::get_column_width, 2))?;
    model.define_method("get_row_height", method!(Model::get_row_height, 2))?;
    model.define_method("set_column_width", method!(Model::set_column_width, 3))?;
    model.define_method("set_row_height", method!(Model::set_row_height, 3))?;
    model.define_method(
        "get_frozen_columns_count",
        method!(Model::get_frozen_columns_count, 1),
    )?;
    model.define_method(
        "get_frozen_rows_count",
        method!(Model::get_frozen_rows_count, 1),
    )?;
    model.define_method(
        "set_frozen_columns_count",
        method!(Model::set_frozen_columns_count, 2),
    )?;
    model.define_method(
        "set_frozen_rows_count",
        method!(Model::set_frozen_rows_count, 2),
    )?;
    model.define_method(
        "get_worksheets_properties",
        method!(Model::get_worksheets_properties, 0),
    )?;
    model.define_method("set_sheet_color", method!(Model::set_sheet_color, 2))?;
    model.define_method("add_sheet", method!(Model::add_sheet, 1))?;
    model.define_method("new_sheet", method!(Model::new_sheet, 0))?;
    model.define_method("delete_sheet", method!(Model::delete_sheet, 1))?;
    model.define_method("rename_sheet", method!(Model::rename_sheet, 2))?;
    model.define_method(
        "get_sheet_dimensions",
        method!(Model::get_sheet_dimensions, 1),
    )?;

    let user_model = module.define_class("UserModel", ruby.class_object())?;
    user_model.define_method("save_to_xlsx", method!(UserModel::save_to_xlsx, 1))?;
    user_model.define_method("save_to_icalc", method!(UserModel::save_to_icalc, 1))?;
    user_model.define_method(
        "apply_external_diffs",
        method!(UserModel::apply_external_diffs, 1),
    )?;
    user_model.define_method("flush_send_queue", method!(UserModel::flush_send_queue, 0))?;
    user_model.define_method("set_user_input", method!(UserModel::set_user_input, 4))?;
    user_model.define_method(
        "get_formatted_cell_value",
        method!(UserModel::get_formatted_cell_value, 3),
    )?;
    user_model.define_method(
        "get_sheet_dimensions",
        method!(UserModel::get_sheet_dimensions, 1),
    )?;
    user_model.define_method("to_bytes", method!(UserModel::to_bytes, 0))?;

    Ok(())
}
