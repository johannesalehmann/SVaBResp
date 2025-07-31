use std::fmt::Formatter;

#[derive(Clone)]
pub struct Identifier<S> {
    pub name: String,
    pub span: S,
}

impl<S> std::fmt::Debug for Identifier<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl<S> Identifier<S> {
    const RESERVED_NAMES: &'static [&'static str] = &[
        "A",
        "bool",
        "clock",
        "const",
        "ctmc",
        "C",
        "double",
        "dtmc",
        "E",
        "endinit",
        "endinvariant",
        "endmodule",
        "endobservables",
        "endrewards",
        "endsystem",
        "false",
        "formula",
        "filter",
        "func",
        "F",
        "global",
        "G",
        "init",
        "invariant",
        "I",
        "int",
        "label",
        "max",
        "mdp",
        "min",
        "module",
        "X",
        "nondeterministic",
        "observable",
        "observables",
        "of",
        "Pmax",
        "Pmin",
        "P",
        "pomdp",
        "popta",
        "probabilistic",
        "prob",
        "pta",
        "rate",
        "rewards",
        "Rmax",
        "Rmin",
        "R",
        "S",
        "stochastic",
        "system",
        "true",
        "U",
        "W",
    ];

    pub fn new<Str: Into<String>>(
        name: Str,
        span: S,
    ) -> Result<Self, crate::identifier::InvalidName> {
        Self::new_with_reserved_option(name, false, span)
    }
    pub fn new_potentially_reserved<Str: Into<String>>(
        name: Str,
        span: S,
    ) -> Result<Self, crate::identifier::InvalidName> {
        Self::new_with_reserved_option(name, true, span)
    }

    fn new_with_reserved_option<Str: Into<String>>(
        name: Str,
        allow_reserved: bool,
        span: S,
    ) -> Result<Self, InvalidName> {
        let name: String = name.into();
        let first_character = name.chars().nth(0);
        match first_character {
            Some(character) => {
                if character.is_ascii_digit() {
                    Err(InvalidName::StartsWithNumber { character })
                } else {
                    Ok(())
                }
            }
            None => Err(InvalidName::Empty),
        }?;

        for (location, character) in name.chars().enumerate() {
            if !(character.is_ascii_alphanumeric() || character == '_') {
                return Err(InvalidName::InvalidCharacter {
                    location,
                    character,
                });
            }
        }

        if !allow_reserved {
            if Self::RESERVED_NAMES.contains(&name.as_str()) {
                return Err(InvalidName::Reserved { name: name.clone() });
            }
        }

        Ok(Self { name, span })
    }
}

impl<S> PartialEq for Identifier<S> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Debug, PartialEq)]
pub enum InvalidName {
    Empty,
    StartsWithNumber { character: char },
    InvalidCharacter { location: usize, character: char },
    Reserved { name: String },
}
