use zelen;

#[test]
fn test_output_parsing_simple_variable() {
    let code = r#"
        var 1..10: x;
        constraint x = 5;
        solve satisfy;
        output ["x = ", show(x), "\n"];
    "#;

    let ast = zelen::parse(code).expect("Failed to parse");
    let model_data = zelen::Translator::translate_with_vars(&ast).expect("Failed to translate");
    
    assert!(!model_data.output_items.is_empty());
    assert_eq!(model_data.output_items.len(), 1);
}

#[test]
fn test_output_parsing_multiple_statements() {
    let code = r#"
        var 1..10: x;
        var 1..10: y;
        constraint x = 2;
        constraint y = 7;
        solve satisfy;
        output ["x = ", show(x), "\n"];
        output ["y = ", show(y), "\n"];
    "#;

    let ast = zelen::parse(code).expect("Failed to parse");
    let model_data = zelen::Translator::translate_with_vars(&ast).expect("Failed to translate");
    
    assert_eq!(model_data.output_items.len(), 2);
}

#[test]
fn test_output_parsing_array() {
    let code = r#"
        array[1..3] of var 1..5: arr;
        constraint arr[1] = 1;
        constraint arr[2] = 2;
        constraint arr[3] = 3;
        solve satisfy;
        output ["arr = ", show(arr), "\n"];
    "#;

    let ast = zelen::parse(code).expect("Failed to parse");
    let model_data = zelen::Translator::translate_with_vars(&ast).expect("Failed to translate");
    
    assert!(!model_data.output_items.is_empty());
}

#[test]
fn test_output_parsing_no_output_statements() {
    let code = r#"
        var 1..10: x;
        constraint x = 5;
        solve satisfy;
    "#;

    let ast = zelen::parse(code).expect("Failed to parse");
    let model_data = zelen::Translator::translate_with_vars(&ast).expect("Failed to translate");
    
    assert!(model_data.output_items.is_empty());
}

#[test]
fn test_output_parsing_string_concatenation() {
    let code = r#"
        var 1..10: x;
        constraint x = 3;
        solve satisfy;
        output ["The answer is ", show(x), " and that's final!\n"];
    "#;

    let ast = zelen::parse(code).expect("Failed to parse");
    let model_data = zelen::Translator::translate_with_vars(&ast).expect("Failed to translate");
    
    assert_eq!(model_data.output_items.len(), 1);
}

#[test]
fn test_output_parsing_complex_model() {
    let code = r#"
        int: n = 4;
        array[1..n] of var 1..n: queens;
        constraint alldifferent(queens);
        solve satisfy;
        output ["queens = ", show(queens), "\n"];
    "#;

    let ast = zelen::parse(code).expect("Failed to parse");
    let model_data = zelen::Translator::translate_with_vars(&ast).expect("Failed to translate");
    
    // Output items should be collected
    assert!(!model_data.output_items.is_empty());
    // queens array should exist in the model
    assert!(model_data.int_var_arrays.contains_key("queens"));
}
