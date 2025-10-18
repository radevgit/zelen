#[test]
fn test_2d_grid_variable_indexing() {
    // Test that 2D array indexing with variable indices works
    let mzn_code = r#"
        int: n = 3;
        int: m = 3;
        array[1..n, 1..m] of var 1..9: grid;
        var 1..n: i;
        var 1..m: j;
        constraint grid[i,j] != grid[1,1];
        solve satisfy;
    "#;
    
    let ast = zelen::parse(&mzn_code)
        .expect("Failed to parse 2D variable indexing test");
    
    let _model = zelen::translate(&ast)
        .expect("Failed to translate 2D variable indexing test");
}

#[test]
fn test_3d_cube_variable_indexing() {
    // Test that 3D array indexing with variable indices works
    let mzn_code = r#"
        int: d1 = 2;
        int: d2 = 2;
        int: d3 = 2;
        array[1..d1, 1..d2, 1..d3] of var 1..8: cube;
        var 1..d1: i;
        var 1..d2: j;
        var 1..d3: k;
        constraint cube[i,j,k] != cube[1,1,1];
        solve satisfy;
    "#;
    
    let ast = zelen::parse(&mzn_code)
        .expect("Failed to parse 3D variable indexing test");
    
    let _model = zelen::translate(&ast)
        .expect("Failed to translate 3D variable indexing test");
}
