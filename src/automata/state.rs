use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

pub static EPSILON: char = 'ε';
pub static EPSILON_TRANSITIONS: &str = "ε*";

// TODO: consider removing Rc trait
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    pub accepting: bool,
    pub transitions: HashMap<char, Vec<Rc<RefCell<State>>>>,
}

impl State {
    pub fn add_transition_for_symbol(&mut self, symbol: char, state: Rc<RefCell<State>>) {
        let t = self.transitions.entry(symbol).or_insert(vec![]);
        t.push(state);
    }

    pub fn get_transition_for_symbol(&self, symbol: &char) -> Option<&Vec<Rc<RefCell<State>>>> {
        self.transitions.get(&symbol)
    }

    // Tests whether this NFA matches the string.
    pub fn test(&self, string: &mut String, visited: &mut HashSet<*const State>) -> bool {
        let ptr = self as *const State;

        if visited.contains(&ptr) {
            return false;
        }

        visited.insert(ptr.to_owned());

        if string.is_empty() {
            if self.accepting {
                return true;
            }

            if let Some(states) = self.get_transition_for_symbol(&EPSILON) {
                for next_state in states.iter() {
                    if next_state.borrow().test(&mut "".to_string(), visited) {
                        return true;
                    }
                }
            }

            return false;
        }

        let mut rest = string.clone();
        let symbol = rest.remove(0);

        if let Some(symbol_transitions) = self.get_transition_for_symbol(&symbol) {
            for next_state in symbol_transitions {
                if next_state.borrow().test(&mut rest, &mut HashSet::new()) {
                    return true;
                }
            }
        }

        if let Some(states) = self.get_transition_for_symbol(&EPSILON) {
            for next_state in states {
                if next_state.borrow().test(string, visited) {
                    return true;
                }
            }
        }

        return false;
    }
}
