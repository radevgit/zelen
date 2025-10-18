//! Tests for enumerated type support

#[cfg(test)]
mod enum_tests {
    use zelen::Translator;

    #[test]
    fn test_enum_definition_parsing() {
        let source = r#"
enum Color = {Red, Green, Blue};
var Color: my_color;
solve satisfy;
"#;
        let model = zelen::parse(source).expect("Failed to parse");
        assert_eq!(model.items.len(), 3); // EnumDef + VarDecl + Solve
    }

    #[test]
    fn test_enum_var_translation() {
        let source = r#"
enum Color = {Red, Green, Blue};
var Color: my_color;
solve satisfy;
"#;
        let _model_data = Translator::translate_with_vars(zelen::parse(source).unwrap())
            .expect("Failed to translate");
        // my_color should be translated as an int var with domain 1..3
    }

    #[test]
    fn test_multiple_enums() {
        let source = r#"
enum Color = {Red, Green, Blue};
enum Size = {Small, Medium, Large};
var Color: color;
var Size: size;
solve satisfy;
"#;
        let model = zelen::parse(source).expect("Failed to parse");
        assert_eq!(model.items.len(), 5); // 2 EnumDefs + 2 VarDecls + Solve
    }

    #[test]
    fn test_enum_with_constraint() {
        let source = r#"
enum Person = {Alice, Bob, Charlie};
var Person: person1;
var Person: person2;
constraint person1 != person2;
solve satisfy;
"#;
        let model_data = Translator::translate_with_vars(zelen::parse(source).unwrap())
            .expect("Failed to translate");
        // Should have 2 integer variables with domain 1..3
        assert_eq!(model_data.int_vars.len(), 2);
    }

    #[test]
    fn test_enum_array() {
        let source = r#"
enum Color = {Red, Green, Blue};
array[1..3] of var Color: colors;
solve satisfy;
"#;
        let model_data = Translator::translate_with_vars(zelen::parse(source).unwrap())
            .expect("Failed to translate");
        // Should have an int var array
        assert_eq!(model_data.int_var_arrays.len(), 1);
        assert_eq!(model_data.int_var_arrays["colors"].len(), 3);
    }
}
