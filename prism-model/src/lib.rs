mod command;
pub use command::{Assignment, Command, Update};

mod expressions;
pub use expressions::{
    DefaultMapExpression, Expression, GlobalVariableReference, IdentityMapExpression,
    MapExpression, VariableScope,
};

mod module;
pub use module::{AddModuleError, Module, ModuleManager, RenameRule, RenameRules, RenamedModule};

mod actions;
pub use actions::{Action, ActionManager, ActionReference, AddActionError};

mod formulas;
pub use formulas::{
    AddFormulaError, CyclicDependency, CyclicDependencyEntry, Formula, FormulaManager,
};

mod labels;
pub use labels::{AddLabelError, Label, LabelManager};

mod model;
pub use model::{Model, ModelType, ModuleExpansionError};

mod rewards;
pub use rewards::{AddRewardsError, Rewards, RewardsElement, RewardsManager, RewardsTarget};

mod variables;
pub use variables::{
    VariableAddError, VariableInfo, VariableManager, VariablePrintingStyle, VariableRange,
    VariableReference,
};

mod identifier;
pub use identifier::{Identifier, InvalidName};

mod properties;
pub use properties::SubstitutableQuery;

use std::fmt::{Display, Formatter};

pub trait Displayable<Ctx>: private::Sealed {
    fn displayable<'a, 'b>(
        &'a self,
        context: &'b Ctx,
    ) -> DisplayableWithContext<'a, 'b, Self, Ctx> {
        DisplayableWithContext {
            element: self,
            context,
        }
    }

    fn fmt_internal(&self, f: &mut Formatter<'_>, context: &Ctx) -> std::fmt::Result;
}

pub struct DisplayableWithContext<'a, 'b, O: ?Sized + Displayable<Ctx>, Ctx> {
    element: &'a O,
    context: &'b Ctx,
}

mod private {
    pub trait Sealed {}
}

impl<O: ?Sized + Displayable<Ctx>, Ctx> Display for DisplayableWithContext<'_, '_, O, Ctx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.element.fmt_internal(f, self.context)
    }
}
