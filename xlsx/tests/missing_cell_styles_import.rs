#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]

use ironcalc::import::load_from_xlsx;

#[test]
// `cellStyles` is optional in `CT_Stylesheet` (ECMA-376 Part 1, 18.8.39: `minOccurs="0"`).
// Before this was handled the importer indexed the first match and panicked with
// `index out of bounds: the len is 0 but the index is 0`.
//
// missing_cell_styles.xlsx is such a file, exported by Jspreadsheet: two empty worksheets
// and a styles.xml with no <cellStyles>. Loading it must succeed.
fn test_styles_without_cell_styles_loads() {
    let model = load_from_xlsx("tests/missing_cell_styles.xlsx", "en", "UTC", "en").unwrap();

    assert_eq!(model.workbook.get_worksheet_names(), ["Sheet1", "Sheet2"]);
    assert!(model.workbook.styles.cell_styles.is_empty());
}
