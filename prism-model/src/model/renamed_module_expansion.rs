use crate::module::AddModuleError;
use crate::variables::MissingVariableRenaming;
use crate::{Identifier, Module};

impl<S: Clone> super::Model<(), Identifier<S>, Identifier<S>, S> {
    pub fn expand_renamed_models(&mut self) -> Result<(), ModuleExpansionError<S>> {
        let renamed_modules = std::mem::replace(&mut self.renamed_modules, Vec::new());
        for renamed_module in renamed_modules {
            let source_index = self
                .modules
                .get_index_by_name(&renamed_module.old_name)
                .ok_or(ModuleExpansionError::RenamingSourceDoesNotExist {
                    old_name: renamed_module.old_name.clone(),
                    new_name: renamed_module.new_name.clone(),
                    renaming_rule: renamed_module.span.clone(),
                })?;
            let source_module = &self.modules.get(source_index).unwrap();
            let module = Module {
                name: renamed_module.new_name.clone(),
                commands: source_module
                    .commands
                    .iter()
                    .map(|c| c.renamed(&renamed_module.rename_rules))
                    .collect(),
                span: renamed_module.span.clone(),
            };
            self.variable_manager
                .add_renamed(
                    source_index,
                    self.modules.modules.len(),
                    &renamed_module.rename_rules,
                )
                .map_err(|e| ModuleExpansionError::MissingVariableRenaming {
                    variable_name: e.variable_name,
                    original_definition: e.original_definition,
                    renaming_rule: renamed_module.span.clone(),
                })?;

            match self.modules.add(module) {
                Ok(_) => Ok(()),
                Err(AddModuleError::ModuleExists { index }) => {
                    Err(ModuleExpansionError::DuplicateModule {
                        name: renamed_module.new_name.clone(),
                        original_module: self.modules.get(index).unwrap().span.clone(),
                        renaming_rule: renamed_module.span.clone(),
                    })
                }
            }?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum ModuleExpansionError<S: Clone> {
    RenamingSourceDoesNotExist {
        old_name: Identifier<S>,
        new_name: Identifier<S>,
        renaming_rule: S,
    },
    MissingVariableRenaming {
        variable_name: Identifier<S>,
        original_definition: S,
        renaming_rule: S,
    },
    DuplicateModule {
        name: Identifier<S>,
        original_module: S,
        renaming_rule: S,
    },
}
