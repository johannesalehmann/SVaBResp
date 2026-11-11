use crate::grouping::StateGroup;
use std::fmt;

pub struct GroupResponsibility {
    pub group: StateGroup,
    pub value: f64,
}

pub struct Responsibility {
    pub responsible: Vec<GroupResponsibility>,
    pub not_responsible: Vec<StateGroup>,
}

impl Responsibility {
    pub fn new(groups: Vec<StateGroup>, values: Vec<f64>) -> Self {
        assert_eq!(groups.len(), values.len());
        let mut responsible = Vec::new();
        let mut not_responsible = Vec::new();
        for (group, value) in groups.into_iter().zip(values) {
            if value == 0.0 {
                not_responsible.push(group);
            } else {
                responsible.push(GroupResponsibility { group, value });
            }
        }
        responsible.sort_by(|a, b| b.value.total_cmp(&a.value));
        Self {
            responsible,
            not_responsible,
        }
    }
}

impl fmt::Display for Responsibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for GroupResponsibility { group, value } in &self.responsible {
            writeln!(f, "{}: {value}", group.name)?;
        }
        write!(
            f,
            "{} groups have no responsibility",
            self.not_responsible.len()
        )
    }
}
