use std::collections::HashMap;

use crate::func::expr::{AtomReplacer, Expr};
use crate::func::Formula;
use crate::helper::error::{CanFail, GenericError};
use crate::model::{GroupedVariables, QModel};
use crate::variables::Variable;
use std::rc::Rc;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum BufferingStrategy {
    AllBuffers,
    Separating,
    Delay,
    Custom,
}

/// Describe buffers between a source component and its targets.
#[derive(Debug)]
enum BufferRef {
    /// No buffering
    Direct,

    /// All targets share the same buffer
    Delayed(BufferSelection),

    /// Each target has its own buffer
    Split(HashMap<usize, usize>),

    /// Custom selection of buffering
    Selected(HashMap<usize, BufferSelection>),
}

#[derive(Default, Debug)]
struct BufferSelection {
    data: Option<usize>,
}

#[derive(Debug)]
pub struct BufferConfig<'a> {
    strategy: BufferingStrategy,
    map: HashMap<usize, BufferRef>,
    model: &'a mut QModel,
    target: Option<usize>,
}

impl BufferSelection {
    fn get_buffer(&mut self, model: &mut QModel, src: usize) -> usize {
        if self.data.is_none() {
            self.data = Some(create_buffer(model, src));
        }
        self.data.unwrap()
    }
}

impl BufferRef {
    fn get_buffer(&mut self, model: &mut QModel, regulator: usize, target: usize) -> Option<usize> {
        match self {
            BufferRef::Direct => None,
            BufferRef::Delayed(bs) => Some(bs.get_buffer(model, regulator)),
            BufferRef::Split(m) => {
                m.entry(target)
                    .or_insert_with(|| create_buffer(model, regulator));
                return m.get(&target).copied();
            }
            BufferRef::Selected(m) => m.get_mut(&target).map(|bs| bs.get_buffer(model, regulator)),
        }
    }

    fn split() -> Self {
        Self::Split(HashMap::new())
    }
    fn delay() -> Self {
        Self::Delayed(BufferSelection::default())
    }
}

impl<'a> BufferConfig<'a> {
    pub fn new(model: &'a mut QModel, strategy: BufferingStrategy) -> Self {
        BufferConfig {
            model,
            strategy,
            map: HashMap::new(),
            target: None,
        }
    }

    fn get_buffer(&mut self, regulator: &Variable) -> Option<usize> {
        if let Some(b) = self.get_buffer_component(regulator.component) {
            Some(self.model.ensure_threshold(b, regulator.value))
        } else {
            None
        }
    }

    fn get_buffer_component(&mut self, regulator: usize) -> Option<usize> {
        let target = self.target?;
        let strategy = self.strategy;
        self.map
            .entry(regulator)
            .or_insert_with(|| match strategy {
                BufferingStrategy::AllBuffers => BufferRef::split(),
                BufferingStrategy::Delay => BufferRef::delay(),
                // FIXME: implement (proper) separating
                BufferingStrategy::Separating => BufferRef::split(),
                BufferingStrategy::Custom => BufferRef::Direct,
            })
            .get_buffer(self.model, regulator, target)
    }

    fn set(&mut self, source: &str, mode: BufferRef) -> CanFail<GenericError> {
        let uid = self.model.get_handle_res(source)?;
        self.map.insert(uid, mode);
        Ok(())
    }

    pub fn split(&mut self, source: &str) -> CanFail<GenericError> {
        self.set(source, BufferRef::split())
    }

    pub fn delay(&mut self, source: &str) -> CanFail<GenericError> {
        self.set(source, BufferRef::delay())
    }

    pub fn direct(&mut self, source: &str) -> CanFail<GenericError> {
        self.set(source, BufferRef::Direct)
    }

    pub fn apply(&mut self) {
        let components: Vec<usize> = self.model.components().copied().collect();
        for cid in components {
            self.set_target(cid);
            let mut rule = self.model.rules.get(cid).unwrap().clone();

            rule.map_assignments(|assign| {
                let expr: Rc<Expr> = assign.formula.convert_as();
                if let Some(e) = expr.replace_variables(self) {
                    assign.formula.set(e);
                }
            });
            // Apply the new rule
            Rc::make_mut(&mut self.model.rules).replace(cid, rule);
        }
    }

    fn set_target(&mut self, target: usize) {
        self.target = Some(target);
    }
}

impl<'a> AtomReplacer for BufferConfig<'a> {
    fn replace(&mut self, varid: usize, value: bool) -> Option<Expr> {
        self.target?;

        let var = match self.model.get_component_value(varid) {
            None => return None,
            Some(v) => *v,
        };

        match self.get_buffer(&var) {
            None => None,
            Some(b) => Some(if value { Expr::ATOM(b) } else { Expr::NATOM(b) }),
        }
    }
}

fn create_buffer(model: &mut QModel, src: usize) -> usize {
    // Create the buffer and add his mirror function
    let buf_id = model.add_component(&format!("_b_{}", model.get_name(src)));

    let variables = model.get_variables(src).clone();
    let mut value = 1;
    for var in variables {
        model.ensure_threshold(buf_id, value);
        model.push_cpt_rule(buf_id, value, Formula::from(Expr::ATOM(var)));
        value += 1;
    }

    buf_id
}
