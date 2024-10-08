use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    automata::nfa_table::NFATable,
    state::{State, EPSILON},
};

#[derive(Debug, Clone, PartialEq)]
pub struct NFA {
    pub in_state: Rc<RefCell<State>>,
    pub out_state: Rc<RefCell<State>>,
}

impl NFA {
    // Tests whether this NFA matches the string.
    fn test(&self, string: &mut String) -> bool {
        self.in_state.borrow().test(string, &mut HashSet::new())
    }

    pub fn empty() -> NFA {
        NFA {
            in_state: Rc::new(RefCell::new(State {
                accepting: false,
                transitions: HashMap::new(),
            })),
            out_state: Rc::new(RefCell::new(State {
                accepting: true,
                transitions: HashMap::new(),
            })),
        }
    }

    // Single char machine.
    pub fn char(symbol: char) -> NFA {
        let in_state = Rc::new(RefCell::new(State {
            accepting: false,
            transitions: HashMap::new(),
        }));

        let out_state = Rc::new(RefCell::new(State {
            accepting: true,
            transitions: HashMap::new(),
        }));

        in_state
            .borrow_mut()
            .add_transition_for_symbol(symbol, out_state.clone());

        NFA {
            in_state,
            out_state,
        }
    }

    // Epsilon machine.
    fn epsilon() -> NFA {
        NFA::char(EPSILON)
    }

    // Creates a concatenation NFA fragment from a single pair of fragments
    fn concat_pair(first: &mut NFA, second: &mut NFA) -> NFA {
        first.out_state.borrow_mut().accepting = false;
        second.out_state.borrow_mut().accepting = true;

        first
            .out_state
            .borrow_mut()
            .add_transition_for_symbol(EPSILON, second.in_state.clone());

        NFA {
            in_state: first.in_state.to_owned(),
            out_state: second.out_state.to_owned(),
        }
    }

    // Creates a concatenation NFA fragment from multiple fragments
    pub fn concat(nfas: Vec<NFA>) -> NFA {
        if nfas.len() < 2 {
            panic!("concat requires at least 2 NFAs to work");
        }

        nfas.iter()
            .cloned()
            .reduce(|mut prev, mut fragment| NFA::concat_pair(&mut prev, &mut fragment))
            .unwrap()
    }

    // Creates a union NFA fragment from a single pair of fragments
    pub fn or_pair(first: &mut NFA, second: &mut NFA) -> NFA {
        let union = NFA {
            in_state: Rc::new(RefCell::new(State {
                accepting: false,
                transitions: HashMap::new(),
            })),
            out_state: Rc::new(RefCell::new(State {
                accepting: true,
                transitions: HashMap::new(),
            })),
        };

        first.out_state.borrow_mut().accepting = false;
        second.out_state.borrow_mut().accepting = false;

        // Create a fork from the union initial state
        // to the two supported fragments.
        union
            .in_state
            .borrow_mut()
            .add_transition_for_symbol(EPSILON, first.in_state.clone());

        union
            .in_state
            .borrow_mut()
            .add_transition_for_symbol(EPSILON, second.in_state.clone());

        // Merge the fork from the two supported fragments
        // into the union end state.
        first
            .out_state
            .borrow_mut()
            .add_transition_for_symbol(EPSILON, union.out_state.clone());

        second
            .out_state
            .borrow_mut()
            .add_transition_for_symbol(EPSILON, union.out_state.clone());

        union
    }

    // Creates a union NFA fragment from multiple fragments
    pub fn or(nfas: Vec<NFA>) -> NFA {
        if nfas.len() < 2 {
            panic!("or requires at least 2 NFAs to work");
        }

        nfas.iter()
            .cloned()
            .reduce(|mut prev, mut fragment| {
                let out = NFA::or_pair(&mut prev, &mut fragment);

                // println!("----or out {:#?}", out);
                out
            })
            .unwrap()
    }

    // Creates a repetition NFA frament (aka Kleene closure).
    pub fn rep(fragment: NFA) -> NFA {
        // Add the skip transition, repeating the machine 0 time.
        fragment
            .in_state
            .borrow_mut()
            .add_transition_for_symbol(EPSILON, fragment.out_state.clone());

        // Add the actual fragment repetition, repeating the machine 1 or more times.
        fragment
            .out_state
            .borrow_mut()
            .add_transition_for_symbol(EPSILON, fragment.in_state.clone());

        NFA {
            in_state: fragment.in_state.to_owned(),
            out_state: fragment.out_state.to_owned(),
        }
    }

    pub fn get_transition_table(&self) -> NFATable {
        NFATable::from(&self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_concatenation(state: &Rc<RefCell<State>>, char: &char) -> Rc<RefCell<State>> {
        let transitions = &state.borrow().transitions;
        assert_eq!(transitions.len(), 1);
        let frag = transitions.get(&char);
        assert_eq!(frag.is_some(), true);
        let states = frag.unwrap();
        assert_eq!(states.len(), 1);

        states[0].to_owned()
    }

    fn assert_union(
        state: &Rc<RefCell<State>>,
        char1: &char,
        char2: &char,
    ) -> (Rc<RefCell<State>>, Rc<RefCell<State>>) {
        let transitions = &state.borrow().transitions;
        assert_eq!(transitions.len(), 1);

        let frag = transitions.get(&EPSILON);
        assert_eq!(frag.is_some(), true);
        let states = frag.unwrap();
        assert_eq!(states.len(), 2);

        let state1 = assert_concatenation(&states[0], char1);
        let state2 = assert_concatenation(&states[1], char2);

        (state1.to_owned(), state2.to_owned())
    }

    #[test]
    fn concat_pair() {
        // The expecetd NFA output from this concatenation should be:
        //
        //  <start> -> a -> ε -> b -> <end>
        //
        let re = NFA::concat_pair(&mut NFA::char('a'), &mut NFA::char('b'));

        // <start> -> a
        let state = assert_concatenation(&re.in_state, &'a');

        // a -> ε
        let state = assert_concatenation(&state, &'ε');

        // ε -> b
        let state = assert_concatenation(&state, &'b');

        // b -> <end>
        let transitions = &state.borrow().transitions;
        assert_eq!(transitions.len(), 0);
    }

    #[test]
    fn concat() {
        // The expecetd NFA output from this concatenation should be:
        //
        //  <start> -> a -> ε -> b -> ε -> c -> <end>
        //
        let re = NFA::concat(vec![NFA::char('a'), NFA::char('b'), NFA::char('c')]);

        // <start> -> a
        let state = assert_concatenation(&re.in_state, &'a');

        // a -> ε
        let state = assert_concatenation(&state, &'ε');

        // ε -> b
        let state = assert_concatenation(&state, &'b');

        // b -> ε
        let state = assert_concatenation(&state, &'ε');

        // ε -> c
        let state = assert_concatenation(&state, &'c');

        // c -> <end>
        let transitions = &state.borrow().transitions;
        assert_eq!(transitions.len(), 0);
    }

    #[test]
    fn or_pair() {
        // The expecetd NFA output from this union should be:
        //
        //             -> ε -> a -> ε -
        //           /                  \
        //  <start> -                    -> <end>
        //           \                  /
        //             -> ε -> b -> ε -
        //
        //
        let re = NFA::or_pair(&mut NFA::char('a'), &mut NFA::char('b'));

        // <start> -> fork into 2 ε framents
        let (state1, state2) = assert_union(&re.in_state, &'a', &'b');

        // a -> ε
        let state1 = assert_concatenation(&state1, &'ε');

        // b -> ε
        let state2 = assert_concatenation(&state2, &'ε');

        // Check the both states merge into the same ε fragment end state.
        assert_eq!(&*state1.borrow(), &*state2.borrow());
        let transitions = &state1.borrow().transitions;
        assert_eq!(transitions.len(), 0);
    }

    #[test]
    fn or() {
        // The expecetd NFA output from this orenation should be:
        //
        //                 -> ε -> a -> ε -
        //               /                  \
        //  <start> ----                     -------> <end>
        //           \   \                  /    /
        //            \    -> ε -> b -> ε -    /
        //             \                     /
        //              \                  /
        //                -> ε -> c -> ε -
        //
        let re = NFA::or(vec![NFA::char('a'), NFA::char('b'), NFA::char('c')]);
        println!("test concat_pair re {:#?}", re);

        // // <start> -> a
        // let state = assert_concatenation(&re.in_state, &'a');
        //
        // // a -> ε
        // let state = assert_concatenation(&state, &'ε');
        //
        // // ε -> b
        // let state = assert_concatenation(&state, &'b');
        //
        // // b -> ε
        // let state = assert_concatenation(&state, &'ε');
        //
        // // ε -> c
        // let state = assert_concatenation(&state, &'c');
        //
        // // c -> <end>
        // let transitions = &state.borrow().transitions;
        // assert_eq!(transitions.len(), 0);
    }
}
