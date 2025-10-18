use std::fs;

#[test]
fn test_2d_grid_parsing() {
    let mzn_code = fs::read_to_string("tests_all/models/test_2d_grid.mzn")
        .expect("Failed to read test_2d_grid.mzn");
    
    let ast = zelen::parse(&mzn_code)
        .expect("Failed to parse 2D grid example");
    
    // Verify the grid variable declaration exists
    let mut found_grid = false;
    for item in &ast.items {
        if let zelen::ast::Item::VarDecl(var_decl) = item {
            if var_decl.name == "grid" {
                found_grid = true;
                
                // Verify it's a multi-dimensional array
                match &var_decl.type_inst {
                    zelen::ast::TypeInst::Array { index_sets, .. } => {
                        // Should have 2 dimensions (n x m)
                        assert_eq!(index_sets.len(), 2, "Grid should be 2-dimensional");
                    }
                    _ => panic!("Grid should be an array type"),
                }
            }
        }
    }
    
    assert!(found_grid, "Grid variable declaration not found");
}

#[test]
fn test_2d_grid_translation() {
    let mzn_code = fs::read_to_string("tests_all/models/test_2d_grid.mzn")
        .expect("Failed to read test_2d_grid.mzn");
    
    let ast = zelen::parse(&mzn_code)
        .expect("Failed to parse 2D grid example");
    
    // Should translate successfully
    let _model = zelen::translate(&ast)
        .expect("Failed to translate 2D grid example");
}

#[test]
fn test_2d_array_indexing() {
    // Test that 2D array indexing with constant indices works
    let mzn_code = r#"
        int: n = 3;
        int: m = 4;
        array[1..n, 1..m] of var 1..9: grid;
        constraint grid[1,1] != grid[2,2];
        solve satisfy;
    "#;
    
    let ast = zelen::parse(&mzn_code)
        .expect("Failed to parse 2D array test");
    
    let _model = zelen::translate(&ast)
        .expect("Failed to translate 2D array indexing test");
}
