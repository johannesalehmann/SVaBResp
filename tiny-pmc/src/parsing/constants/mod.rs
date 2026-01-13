use prism_model_builder::ConstValue;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ConstParsingError {
    InvalidValue { name: String, value: String },
    InvalidAssigment { assignment: String },
}

impl Display for ConstParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error parsing constant: ")?;
        match self {
            ConstParsingError::InvalidValue { name, value } => {
                write!(f, "Invalid value `{}` for constant `{}`", value, name)
            }
            ConstParsingError::InvalidAssigment { assignment } => {
                write!(f, "Invalid assigment `{}`", assignment)
            }
        }
    }
}

pub fn parse_const_assignments(
    assignments: &str,
) -> Result<HashMap<String, ConstValue>, ConstParsingError> {
    let mut result = HashMap::new();

    if assignments.trim().is_empty() {
        return Ok(result);
    }

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
                return Err(ConstParsingError::InvalidValue {
                    name,
                    value: rhs.to_string(),
                });
            };
            result.insert(name, value);
        } else {
            return Err(ConstParsingError::InvalidAssigment {
                assignment: assignment.to_string(),
            });
        }
    }

    Ok(result)
}
