#![allow(clippy::unwrap_used)]

use crate::model::Model;
use crate::test::util::new_empty_model;

#[test]
fn test_datevalue_basic() {
    let mut model = new_empty_model();
    model._set("A1", "=DATEVALUE(\"6-7-25\")");
    model._set("A2", "=DATEVALUE(\"6/7/2025\")");
    model._set("A3", "=DATEVALUE(\"2025-06-07\")");
    model._set("A4", "=DATEVALUE(\"June 7, 2025\")");
    model._set("A5", "=DATEVALUE(\"7 June 25\")");
    model._set("A6", "=DATEVALUE(\"7 Jun 25\")");
    model._set("A7", "=DATEVALUE(\"7 JUNE 2025\")");
    model._set("A8", "=DATEVALUE(\"06-07-2025\")");
    model.evaluate();

    assert_eq!(model._get_text("A1"), *"45815");
    assert_eq!(model._get_text("A2"), *"45815");
    assert_eq!(model._get_text("A3"), *"45815");
    assert_eq!(model._get_text("A4"), *"45815");
    assert_eq!(model._get_text("A5"), *"45815");
    assert_eq!(model._get_text("A6"), *"45815");
    assert_eq!(model._get_text("A7"), *"45815");
    assert_eq!(model._get_text("A8"), *"45815");
}
