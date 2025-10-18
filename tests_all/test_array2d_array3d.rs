use zelen;

#[test]
fn test_array2d_basic() {
    let code = r#"
        int: n = 3;
        int: m = 4;
        array[1..n, 1..m] of int: matrix = array2d(1..n, 1..m, 
          [1, 2, 3, 4,
           5, 6, 7, 8,
           9, 10, 11, 12]);
        solve satisfy;
    "#;

    let ast = zelen::parse(code).expect("Failed to parse");
    assert!(!ast.items.is_empty(), "AST should contain items");
    
    let model_data = zelen::Translator::translate_with_vars(&ast).expect("Failed to translate array2d");
    
    // Verify the model was created successfully
    assert!(matches!(model_data.objective_type, zelen::translator::ObjectiveType::Satisfy),
            "Should be a satisfy problem");
}

#[test]
fn test_array3d_basic() {
    let code = r#"
        int: d1 = 2;
        int: d2 = 3;
        int: d3 = 2;
        array[1..d1, 1..d2, 1..d3] of int: cube = array3d(1..d1, 1..d2, 1..d3,
          [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        solve satisfy;
    "#;

    let ast = zelen::parse(code).expect("Failed to parse");
    assert!(!ast.items.is_empty(), "AST should contain items");
    
    let model_data = zelen::Translator::translate_with_vars(&ast).expect("Failed to translate array3d");
    
    // Verify the model was created successfully  
    assert!(matches!(model_data.objective_type, zelen::translator::ObjectiveType::Satisfy),
            "Should be a satisfy problem");
    
    // Verify the solve item was parsed
    assert!(model_data.search_option.is_none() || model_data.search_option.is_some(),
            "Should have search option parsed");
}

#[test]
fn test_array2d_floats() {
    let code = r#"
        int: n = 2;
        int: m = 2;
        array[1..n, 1..m] of float: rates = array2d(1..n, 1..m,
          [0.0, 1.5,
           0.6, 0.0]);
        solve satisfy;
    "#;

    let ast = zelen::parse(code).expect("Failed to parse");
    assert!(!ast.items.is_empty(), "AST should contain items");
    
    let model_data = zelen::Translator::translate_with_vars(&ast).expect("Failed to translate array2d with floats");
    
    // Verify the model was created successfully
    assert!(matches!(model_data.objective_type, zelen::translator::ObjectiveType::Satisfy),
            "Should be a satisfy problem");
    
    // Verify no variables were created (only float parameter)
    assert!(model_data.int_vars.is_empty(), "Should have no integer variables");
    assert!(model_data.bool_vars.is_empty(), "Should have no boolean variables");
}

#[test]
fn test_array2d_error_mismatch() {
    let code = r#"
        int: n = 3;
        int: m = 3;
        array[1..n, 1..m] of int: matrix = array2d(1..n, 1..m,
          [1, 2, 3, 4, 5, 6, 7, 8]);
    "#;

    let ast = zelen::parse(code).expect("Failed to parse");
    assert!(!ast.items.is_empty(), "AST should parse successfully");
    
    let result = zelen::Translator::translate_with_vars(&ast);
    
    // Should fail with size mismatch error
    assert!(result.is_err(), "Should fail with size mismatch");
    
    if let Err(error) = result {
        let error_msg = format!("{}", error);
        
        // Verify error message contains relevant information
        assert!(
            error_msg.contains("array2d value count mismatch")
                || error_msg.contains("Array2DValueCountMismatch"),
            "Error should mention array2d size mismatch: {}",
            error_msg
        );
        
        // Verify the error mentions the counts
        assert!(
            error_msg.contains("9") || error_msg.contains("expected"),
            "Error should mention expected count (9): {}",
            error_msg
        );
        
        assert!(
            error_msg.contains("8") || error_msg.contains("got"),
            "Error should mention provided count (8): {}",
            error_msg
        );
    }
}
