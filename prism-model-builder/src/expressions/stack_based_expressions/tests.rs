use super::*;
use crate::expressions::VariableType;

struct MockValueSource {}

impl MockValueSource {
    pub fn new() -> Self {
        Self {}
    }
}

impl ValuationSource for MockValueSource {
    fn get_int(&self, index: VariableReference) -> i64 {
        let _ = index;
        panic!("Mock value source does not provide any values");
    }

    fn get_bool(&self, index: VariableReference) -> bool {
        let _ = index;
        panic!("Mock value source does not provide any values");
    }

    fn get_float(&self, index: VariableReference) -> f64 {
        let _ = index;
        panic!("Mock value source does not provide any values");
    }

    fn get_type(&self, index: VariableReference) -> VariableType {
        let _ = index;
        panic!("Mock value source does not provide any types");
    }
}

#[test]
fn const_int() {
    let expr = StackBasedExpression::from_expression(
        &Expression::int(12).minus(Expression::float(3.4)),
        &VariableManager::new(),
    );
    let value = expr.evaluate_as_float(&MockValueSource::new());
    assert_eq!(value, 12.0 - 3.4);
}
