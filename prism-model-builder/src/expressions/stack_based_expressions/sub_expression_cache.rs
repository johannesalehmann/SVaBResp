#[derive(Copy, Clone, PartialEq)]
enum CachedSubExpressionValue {
    Unknown,
    Int(i64),
    Float(f64),
    Bool(bool),
}

pub struct SubExpressionCache {
    values: Vec<CachedSubExpressionValue>,
}

impl SubExpressionCache {
    pub fn new(size: usize) -> Self {
        Self {
            values: vec![CachedSubExpressionValue::Unknown; size],
        }
    }

    pub fn clear(&mut self) {
        for value in &mut self.values {
            *value = CachedSubExpressionValue::Unknown;
        }
    }

    pub fn get_int(&self, index: usize) -> Option<i64> {
        match self.values[index] {
            CachedSubExpressionValue::Unknown => None,
            CachedSubExpressionValue::Int(val) => Some(val),
            _ => panic!(
                "Sub-expression cache contains a value for this sub expression, but it is not an int."
            ),
        }
    }

    pub fn get_bool(&self, index: usize) -> Option<bool> {
        match self.values[index] {
            CachedSubExpressionValue::Unknown => None,
            CachedSubExpressionValue::Bool(val) => Some(val),
            _ => panic!(
                "Sub-expression cache contains a value for this sub expression, but it is not a bool."
            ),
        }
    }

    pub fn get_float(&self, index: usize) -> Option<f64> {
        match self.values[index] {
            CachedSubExpressionValue::Unknown => None,
            CachedSubExpressionValue::Float(val) => Some(val),
            _ => panic!(
                "Sub-expression cache contains a value for this sub expression, but it is not a float."
            ),
        }
    }

    pub fn store_int(&mut self, index: usize, val: i64) {
        self.values[index] = CachedSubExpressionValue::Int(val);
    }

    pub fn store_bool(&mut self, index: usize, val: bool) {
        self.values[index] = CachedSubExpressionValue::Bool(val);
    }

    pub fn store_float(&mut self, index: usize, val: f64) {
        self.values[index] = CachedSubExpressionValue::Float(val);
    }
}
