// Test: Use translate() instead of translate_with_vars()

use zelen::parse;
use zelen::translator::Translator;

fn main() {
    let source = r#"
        var 1..100: dividend;
        var 1..10: divisor;
        var 0..9: remainder;
        
        constraint remainder == dividend mod divisor;
        constraint dividend == 47;
        constraint divisor == 10;
        
        solve satisfy;
    "#;

    match parse(source) {
        Ok(ast) => {
            match Translator::translate(&ast) {
                Ok(model) => {
                    match model.solve() {
                        Ok(_solution) => {
                            println!("✓ Solution!");
                        }
                        Err(e) => {
                            println!("✗ No solution with translate(): {:?}", e);
                        }
                    }
                }
                Err(e) => println!("✗ Translation error: {:?}", e),
            }
        }
        Err(e) => println!("✗ Parse error: {:?}", e),
    }
}
