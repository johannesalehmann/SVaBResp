use prism_model_builder::ConstValue;
use std::collections::HashMap;

pub fn parse_const_assignments(assignments: &str) -> HashMap<String, ConstValue> {
    let mut result = HashMap::new();

    for assignment in assignments.split(";") {
        if let Some((lhs, rhs)) = assignment.split_once("=") {
            let name = lhs.trim().to_string();
            let value = if let Ok(i) = rhs.parse::<i64>() {
                ConstValue::Int(i)
            } else if let Ok(f) = rhs.parse::<f64>() {
                ConstValue::Float(f)
            } else if let Ok(b) = rhs.parse::<bool>() {
                ConstValue::Bool(b)
            } else {
                panic!("Cannot parse value {} for constant {}", rhs, name);
            };
            result.insert(name, value);
        } else {
            panic!("Cannot parse constant assignment {}", assignment);
        }
    }

    result
}
