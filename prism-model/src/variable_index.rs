pub struct VariableIndex {
    index: usize,
}

impl VariableIndex {
    pub fn local(index: usize) -> Self {
        if index >= usize::MAX - 1 {
            panic!(
                "Cannot create local variable index for module {}, as that index is reserved.",
                index
            )
        }
        Self { index }
    }

    pub fn global_var() -> Self {
        Self {
            index: usize::MAX - 1,
        }
    }

    pub fn global_const() -> Self {
        Self { index: usize::MAX }
    }
}
