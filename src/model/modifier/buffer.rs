use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::func::expr::{AtomReplacer, Expr};
use crate::func::Formula;
use crate::model::QModel;
use crate::model::SharedComponent;

#[derive(PartialEq, Clone, Copy)]
pub enum BufferingStrategy {
    ALLBUFFERS,
    SEPARATING,
    DELAY,
    CUSTOM,
}

/// Describe buffers between a source components and its targets.
enum BufferRef {
    SKIP,
    DELAY(BufferSelection),
    SEPARATED(HashMap<usize, BufferSelection>),
    SELECT(HashMap<usize, Rc<Option<usize>>>),
}

struct BufferSelection {
    data: Rc<RefCell<Option<usize>>>,
}

pub struct BufferConfig<'a> {
    strategy: BufferingStrategy,
    map: HashMap<usize, BufferRef>,
    model: &'a mut dyn QModel,
}

impl BufferSelection {
    fn new() -> Self {
        BufferSelection {
            data: Rc::new(RefCell::new(None)),
        }
    }

    fn get_buffer(&self, model: &mut dyn QModel, src: usize) -> usize {
        let cell = self.data.as_ref();
        if cell.borrow().is_none() {
            let v = create_buffer(model, src);
            self.data.as_ref().replace(Some(v));
        }
        return cell.borrow().unwrap();
    }
}

impl<'a> BufferConfig<'a> {
    pub fn new(model: &'a mut dyn QModel, strategy: BufferingStrategy) -> Self {
        BufferConfig {
            model: model,
            strategy: strategy,
            map: HashMap::new(),
        }
    }

    pub fn add_single_buffer_by_name(&mut self, source: &str, target: &str) {
        let usrc = self.model.component_by_name(source);
        if usrc.is_none() {
            println!("unknown buffering source: {}", source);
            return;
        }
        let utgt = self.model.component_by_name(target);
        if utgt.is_none() {
            println!("unknown buffering source: {}", target);
            return;
        }

        self.add_single_buffer(usrc.unwrap(), utgt.unwrap());
    }

    pub fn add_single_buffer(&mut self, source: usize, target: usize) {
        if self.strategy != BufferingStrategy::CUSTOM {
            panic!("Only custom bufferings allow to add buffers manually")
        }
        unimplemented!()
    }

    pub fn add_multiple_buffers(&mut self, source: usize, targets: &[usize]) {
        if self.strategy != BufferingStrategy::CUSTOM {
            panic!("Only custom bufferings allow to add buffers manually")
        }
        unimplemented!()
    }

    pub fn add_delay_by_name(&mut self, source: &str) {
        let usrc = self.model.component_by_name(source);
        if usrc.is_none() {
            println!("unknown buffering source: {}", source);
            return;
        }
        self.add_delay(usrc.unwrap());
    }

    pub fn add_delay(&mut self, source: usize) {
        if self.strategy != BufferingStrategy::CUSTOM {
            panic!("Only custom bufferings allow to add buffers manually")
        }
        unimplemented!()
    }

    pub fn apply(&mut self) {
        let components: Vec<(usize, SharedComponent)> = self.model.components_copy();
        for (cid, component) in components {
            let mut cpt = component.borrow_mut();
            for assign in cpt.assignments.iter_mut() {
                let expr: Rc<Expr> = assign.formula.convert_as();
                let new_expr = expr.replace_variables(self);
                if let Some(e) = new_expr {
                    assign.formula.set(e);
                }
            }
        }
    }

    fn get_buffer_ref(&mut self, src: usize) -> &BufferRef {
        // ensure that the map has a matching entry
        if !self.map.contains_key(&src) {
            self.map.insert(
                src,
                match self.strategy {
                    BufferingStrategy::CUSTOM => BufferRef::SKIP,
                    BufferingStrategy::DELAY => BufferRef::DELAY(BufferSelection::new()),
                    BufferingStrategy::ALLBUFFERS => BufferRef::SEPARATED(HashMap::new()),
                    BufferingStrategy::SEPARATING => BufferRef::SEPARATED(HashMap::new()),
                },
            );
        }

        return self.map.get(&src).unwrap();
    }
}

impl<'a> AtomReplacer for BufferConfig<'a> {
    fn ask_buffer(&mut self, varid: usize, value: bool) -> Option<Expr> {
        let var = self.model.get_variable(varid);

        // FIXME: grab the buffer for the source component and replace if needed

        None
    }
}

fn create_buffer(model: &mut dyn QModel, src: usize) -> usize {
    let cmp = model.get_component_ref(src);
    // Create the buffer and add his mirror function
    let buf_id = model.add_component("buffer");

    for (value, var) in cmp.borrow().variables() {
        let buf_var = model.ensure_associated_variable(buf_id, *value);
        model.set_rule(buf_var, Formula::from(Expr::ATOM(*var)));
    }

    buf_id
}

fn replaced_variable(var: usize) -> usize {
    var
}
