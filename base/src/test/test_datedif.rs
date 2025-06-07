#![allow(clippy::unwrap_used)]

use crate::model::Model;
use crate::test::util::new_empty_model;

#[test]
fn test_datedif_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=DATEDIF(\"2025-01-01\", \"2025-01-03\", \"d\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"2");
}

#[test]
fn test_datedif_invalid_unit() {
    let mut model = new_empty_model();
    model._set("A1", "=DATEDIF(\"2025-01-01\", \"2025-01-03\", \"Y\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"Invalid unit");
}
