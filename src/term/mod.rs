mod eq;
mod index;
mod map_reference;
mod normalize;
#[cfg(feature = "parser")]
mod parse;
mod show;
mod stratified;

pub use crate::analysis::{Definitions, TypedDefinitions};
pub use normalize::NormalizationError;
#[cfg(feature = "parser")]
pub use parse::{parse, typed, untyped, ParseError};
use serde::{Deserialize, Serialize};
pub(crate) use show::debug_reference;
pub use show::Show;
pub use stratified::{StratificationError, Stratified};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(transparent)]
pub struct Index(pub usize);

#[derive(Serialize, Deserialize, Clone)]
pub enum Term<T> {
    // Untyped language
    Variable(Index),
    Lambda {
        body: Box<Term<T>>,
        erased: bool,
    },
    Apply {
        function: Box<Term<T>>,
        argument: Box<Term<T>>,
        erased: bool,
    },
    Put(Box<Term<T>>),
    Duplicate {
        expression: Box<Term<T>>,
        body: Box<Term<T>>,
    },
    Reference(T),

    // Typed extensions
    Universe,
    Function {
        argument_type: Box<Term<T>>,
        return_type: Box<Term<T>>,
        erased: bool,
    },
    Annotation {
        checked: bool,
        expression: Box<Term<T>>,
        ty: Box<Term<T>>,
    },
    Wrap(Box<Term<T>>),
}
