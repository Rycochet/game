use serde::Deserialize;

use super::*;

#[derive(Debug, Deserialize, PartialEq)]
struct Where {
    #[serde(default)]
    x: i64,
}

#[test]
fn params_default_to_none_and_report_bad_ones() {
    assert_eq!(params::<Where>(None).unwrap(), Where { x: 0 });
    assert_eq!(
        params::<Where>(Some(json!({"x": 3}))).unwrap(),
        Where { x: 3 }
    );
    let error = params::<Where>(Some(json!({"x": "three"}))).unwrap_err();
    assert_eq!(error.code, error_codes::INVALID_PARAMS);
}

#[test]
fn methods_wait_until_the_api_has_started() {
    let mut app = App::new();
    app.add_debug_method("test.answer", |_: In<Option<Value>>| Ok(json!(42)));
    // Not started: nothing to put them in, and nothing goes wrong.
    register_methods(&mut app);
    assert!(!app.world().contains_resource::<RemoteMethods>());

    let mut app = App::new();
    app.add_debug_method("test.answer", |_: In<Option<Value>>| Ok(json!(42)));
    app.insert_resource(RemoteMethods::new());
    register_methods(&mut app);
    let methods = app.world().resource::<RemoteMethods>();
    assert!(methods.get("test.answer").is_some());
}
