use std::hash::Hasher;

#[derive(Copy, Clone, PartialEq)]
enum Value {
    Int(i64),
    Bool(bool),
    Float(f64),
}

impl std::hash::Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Int(i) => {
                123.hash(state);
                i.hash(state)
            }
            Value::Bool(b) => {
                456.hash(state);
                b.hash(state)
            }
            Value::Float(f) => {
                789.hash(state);
                ((f * 10000.0) as i64).hash(state) // TODO: This is not a good hash implementation
            }
        }
    }
}

impl Eq for Value {}

impl Value {
    fn as_int(self) -> i64 {
        match self {
            Self::Int(val) => val,
            _ => panic!("Value has incorrect type"),
        }
    }
    fn as_bool(self) -> bool {
        match self {
            Self::Bool(val) => val,
            _ => panic!("Value has incorrect type"),
        }
    }
    fn as_float(self) -> f64 {
        match self {
            Self::Float(val) => val,
            _ => panic!("Value has incorrect type"),
        }
    }
}

#[derive(Eq, PartialEq, Clone, Hash)]
pub struct ValuationVector {
    values: Vec<Value>,
}

impl super::Valuation for ValuationVector {
    type ContextType = Context;
    type ContextBuilderType = ContextBuilder;
    type BuilderType = ValuationBuilder;

    fn get_context_builder() -> Self::ContextBuilderType {
        ContextBuilder::new()
    }

    fn get_builder(context: &Self::ContextType) -> Self::BuilderType {
        ValuationBuilder::new(context.number_of_variables)
    }

    fn evaluate_bounded_int(&self, index: usize) -> i64 {
        self.values[index].as_int()
    }

    fn evaluate_bool(&self, index: usize) -> bool {
        self.values[index].as_bool()
    }

    fn evaluate_unbounded_int(&self, index: usize) -> i64 {
        self.values[index].as_int()
    }

    fn evaluate_float(&self, index: usize) -> f64 {
        self.values[index].as_float()
    }
    fn set_bounded_int(&mut self, index: usize, value: i64) {
        self.values[index] = Value::Int(value);
    }
    fn set_bool(&mut self, index: usize, value: bool) {
        self.values[index] = Value::Bool(value);
    }
    fn set_unbounded_int(&mut self, index: usize, value: i64) {
        self.values[index] = Value::Int(value);
    }
    fn set_float(&mut self, index: usize, value: f64) {
        self.values[index] = Value::Float(value);
    }
}
pub struct Context {
    number_of_variables: usize,
}

pub struct ContextBuilder {
    context: Context,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            context: Context {
                number_of_variables: 0,
            },
        }
    }
}

impl super::ContextBuilder<Context> for ContextBuilder {
    fn register_bounded_int(&mut self, _min: i64, _max: i64) {
        self.context.number_of_variables += 1;
    }

    fn register_bool(&mut self) {
        self.context.number_of_variables += 1;
    }

    fn register_unbounded_int(&mut self) {
        self.context.number_of_variables += 1;
    }

    fn register_float(&mut self) {
        self.context.number_of_variables += 1;
    }

    fn finish(self) -> Context {
        self.context
    }
}

pub struct ValuationBuilder {
    values: Vec<Value>,
}

impl ValuationBuilder {
    pub fn new(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }
}

impl super::ValuationBuilder<ValuationVector> for ValuationBuilder {
    fn add_bounded_int(&mut self, value: i64) {
        self.values.push(Value::Int(value));
    }

    fn add_bool(&mut self, value: bool) {
        self.values.push(Value::Bool(value));
    }

    fn add_int(&mut self, value: i64) {
        self.values.push(Value::Int(value));
    }

    fn add_float(&mut self, value: f64) {
        self.values.push(Value::Float(value));
    }

    fn finish(self) -> ValuationVector {
        ValuationVector {
            values: self.values,
        }
    }
}
