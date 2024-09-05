use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::nfa::NFA;

#[derive(Debug, Clone)]
pub struct DFA {
    pub nfa: NFA,
}

impl DFA {
    // DFA alphabet. Same as in NFA, except ε.
    pub fn get_alphabet() -> HashSet<String> {
        todo!()
    }

    // Accepting states (calculated during table built).
    pub fn get_accepting_states() -> HashSet<String> {
        todo!()
    }

    // DFA transition table built from NFA table.
    pub fn get_transition_table(&self) -> HashMap<String, HashMap<String, Vec<usize>>> {
        crate::dfa_table::DFATableBuilder::build_table(&self.nfa)
    }

    // Tests whether this DFA accepts the string.
    fn test(&self, string: &mut String) -> bool {
        todo!()
    }
}
