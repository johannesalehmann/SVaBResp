// TODO: While this is useful to have, it is currently poorly integrated into the model parser.
//  Consider adapting the Span type to include line information instead.
pub struct CharacterToLineMap {
    line_start_indices: Vec<usize>,
}

impl CharacterToLineMap {
    pub fn new() -> Self {
        Self {
            line_start_indices: Vec::new(),
        }
    }

    pub fn from_str<S: AsRef<str>>(source: S) -> Self {
        let mut res = Self::new();
        res.add_line(0);
        for (index, char) in source.as_ref().chars().enumerate() {
            if char == '\n' {
                res.add_line(index + 1);
            }
        }

        res
    }

    pub fn add_line(&mut self, line_start_index: usize) {
        if let Some(&last_line_start_index) = self.line_start_indices.last() {
            if last_line_start_index >= line_start_index {
                panic!(
                    "Cannot add a line start index that is less or equal to the previous line start index);"
                )
            }
        }
        self.line_start_indices.push(line_start_index);
    }

    pub fn get_line(&self, char_index: usize) -> usize {
        self.line_start_indices
            .partition_point(|line_start| *line_start <= char_index)
    }
}
