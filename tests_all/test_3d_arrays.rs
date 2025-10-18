use std::fs;

#[test]
fn test_3d_cube_parsing() {
    let mzn_code = fs::read_to_string("tests/models/test_3d_cube.mzn")
        .expect("Failed to read test_3d_cube.mzn");
    
    let ast = zelen::parse(&mzn_code)
        .expect("Failed to parse 3D cube example");
    
    // Verify the cube variable declaration exists
    let mut found_cube = false;
    for item in &ast.items {
        if let zelen::ast::Item::VarDecl(var_decl) = item {
            if var_decl.name == "cube" {
                found_cube = true;
                
                // Verify it's a 3-dimensional array
                match &var_decl.type_inst {
                    zelen::ast::TypeInst::Array { index_sets, .. } => {
                        // Should have 3 dimensions
                        assert_eq!(index_sets.len(), 3, "Cube should be 3-dimensional");
                    }
                    _ => panic!("Cube should be an array type"),
                }
            }
        }
    }
    
    assert!(found_cube, "Cube variable declaration not found");
}

#[test]
fn test_3d_cube_translation() {
    let mzn_code = fs::read_to_string("tests/models/test_3d_cube.mzn")
        .expect("Failed to read test_3d_cube.mzn");
    
    let ast = zelen::parse(&mzn_code)
        .expect("Failed to parse 3D cube example");
    
    // Should translate successfully
    let _model = zelen::translate(&ast)
        .expect("Failed to translate 3D cube example");
}

#[test]
fn test_3d_array_indexing() {
    // Test that 3D array indexing with constant indices works
    let mzn_code = r#"
        int: d1 = 2;
        int: d2 = 2;
        int: d3 = 2;
        array[1..d1, 1..d2, 1..d3] of var 1..8: data;
        constraint data[1,1,1] != data[2,2,2];
        constraint data[1,2,1] < data[2,1,2];
        solve satisfy;
    "#;
    
    let ast = zelen::parse(&mzn_code)
        .expect("Failed to parse 3D array test");
    
    let _model = zelen::translate(&ast)
        .expect("Failed to translate 3D array indexing test");
}
