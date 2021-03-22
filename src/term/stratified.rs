use super::{normalize::NormalizationError, Definitions, Term};

#[derive(Debug, Clone)]
pub struct Stratified<'a, T: Definitions>(pub(crate) Term, pub(crate) &'a T);

impl<'a, T: Definitions> Stratified<'a, T> {
    pub fn normalize(&mut self) -> Result<(), NormalizationError> {
        self.0.normalize(self.1)
    }

    pub fn into_inner(self) -> Term {
        self.0
    }
}

#[derive(Debug)]
pub enum StratificationError {
    AffineReused { name: String, term: Term },
    AffineUsedInBox { name: String, term: Term },
    DupNonUnitBoxMultiplicity { name: String, term: Term },
    UndefinedReference { name: String },
}

impl Term {
    fn uses(&self) -> usize {
        fn uses_helper(term: &Term, depth: usize) -> usize {
            use Term::*;
            match term {
                Symbol(symbol) => {
                    if symbol.0 == depth {
                        1
                    } else {
                        0
                    }
                }
                Reference(_) => 0,
                Lambda { body, .. } => uses_helper(body, depth + 1),
                Apply { function, argument } => {
                    uses_helper(function, depth) + uses_helper(argument, depth)
                }
                Put(term) => uses_helper(term, depth),
                Duplicate {
                    expression, body, ..
                } => uses_helper(expression, depth) + uses_helper(body, depth + 1),
                Universe => 0,
                Wrap(term) => uses_helper(term, depth),
                Annotation { expression, .. } => uses_helper(expression, depth),
                Function {
                    return_type,
                    argument_type,
                    ..
                } => uses_helper(return_type, depth + 1) + uses_helper(argument_type, depth),
            }
        }

        uses_helper(self, 0)
    }

    fn is_at_level(&self, target_level: usize, depth: usize, level: usize) -> bool {
        use Term::*;

        match self {
            Reference(_) => true,
            Symbol(symbol) => symbol.0 != depth || level == target_level,
            Lambda { body, .. } => body.is_at_level(target_level, depth + 1, level),
            Apply { function, argument } => {
                function.is_at_level(target_level, depth, level)
                    && argument.is_at_level(target_level, depth, level)
            }
            Put(term) => term.is_at_level(target_level, depth, level + 1),
            Wrap(term) => term.is_at_level(target_level, depth, level),
            Annotation { expression, .. } => expression.is_at_level(target_level, depth, level),
            Duplicate {
                expression, body, ..
            } => {
                expression.is_at_level(target_level, depth, level)
                    && body.is_at_level(target_level, depth + 1, level)
            }
            Universe => true,
            Function {
                argument_type,
                return_type,
                ..
            } => {
                argument_type.is_at_level(target_level, depth, level)
                    && return_type.is_at_level(target_level, depth + 1, level)
            }
        }
    }

    fn is_stratified<T: Definitions>(&self, definitions: &T) -> Result<(), StratificationError> {
        use Term::*;

        match &self {
            Lambda { body, binding } => {
                if body.uses() > 1 {
                    return Err(StratificationError::AffineReused {
                        name: binding.clone(),
                        term: self.clone(),
                    });
                }
                if !body.is_at_level(0, 0, 0) {
                    return Err(StratificationError::AffineUsedInBox {
                        name: binding.clone(),
                        term: self.clone(),
                    });
                }
                body.is_stratified(definitions)?;
            }
            Apply { function, argument } => {
                function.is_stratified(definitions)?;
                argument.is_stratified(definitions)?;
            }
            Put(term) => {
                term.is_stratified(definitions)?;
            }
            Wrap(term) => {
                term.is_stratified(definitions)?;
            }
            Annotation { expression, .. } => {
                expression.is_stratified(definitions)?;
            }
            Function {
                argument_type,
                return_type,
                ..
            } => {
                argument_type.is_stratified(definitions)?;
                return_type.is_stratified(definitions)?;
            }
            Duplicate {
                binding,
                body,
                expression,
            } => {
                if !body.is_at_level(1, 0, 0) {
                    return Err(StratificationError::DupNonUnitBoxMultiplicity {
                        name: binding.clone(),
                        term: self.clone(),
                    });
                }
                expression.is_stratified(definitions)?;
                body.is_stratified(definitions)?;
            }
            Reference(name) => {
                if let Some(term) = definitions.get(name) {
                    term.is_stratified(definitions)?;
                } else {
                    return Err(StratificationError::UndefinedReference { name: name.clone() });
                }
            }
            Symbol(_) | Universe => {}
        }

        Ok(())
    }

    pub fn stratified<T: Definitions>(
        self,
        definitions: &T,
    ) -> Result<Stratified<'_, T>, StratificationError> {
        println!("normed: {:?}", {
            let mut item = self.clone();
            item.normalize(definitions).unwrap();
            item
        });
        self.is_stratified(definitions)?;
        Ok(Stratified(self, definitions))
    }
}
