use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

// TODO: consider removing Rc+RefCell trates
#[derive(Debug, Clone, PartialEq, Eq)]
struct State {
    accepting: bool,
    transitions: HashMap<char, Vec<Rc<RefCell<State>>>>,
}

impl State {
    fn add_transition_for_symbol(&mut self, symbol: char, state: Rc<RefCell<State>>) {
        let t = self.transitions.entry(symbol).or_insert(vec![]);
        t.push(state);
    }

    fn get_transition_for_symbol(&self, symbol: &char) -> Option<&Vec<Rc<RefCell<State>>>> {
        self.transitions.get(&symbol)
    }

    // Tests whether this NFA matches the string.
    fn test(&self, string: &mut String, visited: &mut HashSet<*const State>) -> bool {
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

#[derive(Debug, Clone)]
pub struct NFA {
    in_state: Rc<RefCell<State>>,
    out_state: Rc<RefCell<State>>,
}

static EPSILON: char = 'ε';

impl NFA {
    // Tests whether this NFA matches the string.
    fn test(&self, string: &mut String) -> bool {
        self.in_state.borrow().test(string, &mut HashSet::new())
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
    fn or_pair(first: &mut NFA, second: &mut NFA) -> NFA {
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

        // Create a fork from the union incoming state
        // to the two supported framents.
        union
            .in_state
            .borrow_mut()
            .add_transition_for_symbol(EPSILON, first.in_state.clone());

        union
            .in_state
            .borrow_mut()
            .add_transition_for_symbol(EPSILON, second.in_state.clone());

        // Merge the frok from the two supported framents
        // into the union outgoing state.
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
    fn or(nfas: Vec<NFA>) -> NFA {
        if nfas.len() < 2 {
            panic!("or requires at least 2 NFAs to work");
        }

        nfas.iter()
            .cloned()
            .reduce(|mut prev, mut fragment| {
                let out = NFA::or_pair(&mut prev, &mut fragment);

                println!("----or out {:#?}", out);
                out
            })
            .unwrap()
    }

    // Creates a repetition NFA frament (aka Kleene closure).
    fn rep(fragment: NFA) -> NFA {
        todo!()
    }

    fn get_transition_table(&self) -> HashMap<usize, HashMap<String, Vec<usize>>> {
        // Epsilon closure (denotaed as "ε*"): set of states reacheable from this state,
        // following only ε-transitions. Which means, it containes the state it self and
        // the states is moving to.
        // {
        //   '1': { a: [ 2 ], 'ε*': [ 1 ] },
        //   '2': { 'ε*': [ 2, 3 ] },
        //   '3': { b: [ 4 ], 'ε*': [ 3 ] },
        //   '4': { 'ε*': [ 4 ] },
        // }
        //
        //  {
        //     in_state: RefCell {
        //         value: State {
        //             accepting: false,
        //             transitions: {
        //                 'a': [
        //                     RefCell {
        //                         value: State {
        //                             accepting: false,
        //                             transitions: {
        //                                 'ε': [
        //                                     RefCell {
        //                                         value: State {
        //                                             accepting: false,
        //                                             transitions: {
        //                                                 'b': [
        //                                                     RefCell {
        //                                                         value: State {
        //                                                             accepting: true,
        //                                                             transitions: {},
        //                                                         },
        //                                                     },
        //                                                 ],
        //                                             },
        //                                         },
        //                                     },
        //                                 ],
        //                             },
        //                         },
        //                     },
        //                 ],
        //             },
        //         },
        //     },
        //     out_state: RefCell {
        //         value: State {
        //             accepting: true,
        //             transitions: {},
        //         },
        //     },
        // }
        let mut table: HashMap<usize, HashMap<String, Vec<usize>>> = HashMap::new();

        let mut state_count: usize = 0;
        let mut map_state_ids = HashMap::new();

        let mut get_state_id = |state: &State| -> usize {
            let ptr = state as *const State;

            let state_id = map_state_ids.entry(ptr).or_insert_with(|| {
                state_count += 1;
                state_count
            });

            *state_id
        };

        let mut walk_states = vec![self.in_state.to_owned()];

        while walk_states.len() > 0 {
            let ref_state = walk_states.remove(0);
            let state = &*ref_state.borrow();
            let state_id = get_state_id(state);

            let mut row: HashMap<String, Vec<usize>> = HashMap::new();
            row.insert("ε*".to_string(), vec![state_id]);

            for (t, states) in &state.transitions {
                let transition_label = match t {
                    'ε' => "ε*".to_string(),
                    c => c.to_string(),
                };

                let ids = row.entry(transition_label).or_insert(vec![]);

                for child_state in states {
                    let child_state_id = get_state_id(&child_state.borrow());
                    ids.push(child_state_id);
                    walk_states.push(child_state.to_owned());
                }
            }

            table.insert(state_id, row.to_owned());
        }

        table
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
        // The expecetd NFA output from this orenation should be:
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

    #[test]
    fn get_transition_table_concat() {
        // Given regex /ab/
        // Its graph looks like:
        //                    a          ε          b
        //  (s:1 - starting) ---> (s:2) ---> (s:3) ---> (s:4 - accepting)
        //
        // Its NFA table is:
        //
        // ┌─────┬───┬───┬───────┐
        // │     │ a │ b │ ε*    │
        // ├─────┼───┼───┼───────┤
        // │ 1 > │ 2 │   │ 1     │
        // ├─────┼───┼───┼───────┤
        // │ 2   │   │   │ {2,3} │
        // ├─────┼───┼───┼───────┤
        // │ 3   │   │ 4 │ 3     │
        // ├─────┼───┼───┼───────┤
        // │ 4 ✓ │   │   │ 4     │
        // └─────┴───┴───┴───────┘
        //
        // And the data representation as JSON is:
        //
        // {
        //   '1': { a: [ 2 ], 'ε*': [ 1 ] },
        //   '2': { 'ε*': [ 2, 3 ] },
        //   '3': { b: [ 4 ], 'ε*': [ 3 ] },
        //   '4': { 'ε*': [ 4 ] },
        // }
        //  {
        //     in_state: RefCell {                                                          -> state:1 - starting
        //         value: State {
        //             accepting: false,
        //             transitions: {
        //                 'a': [                                                           -> transition:a
        //                     RefCell {
        //                         value: State {                                           -> state:2
        //                             accepting: false,
        //                             transitions: {
        //                                 'ε': [                                           -> transition:ε
        //                                     RefCell {
        //                                         value: State {                           -> state:3
        //                                             accepting: false,
        //                                             transitions: {
        //                                                 'b': [                           -> transition:b
        //                                                     RefCell {
        //                                                         value: State {           -> state:4 - accepting
        //                                                             accepting: true,
        //                                                             transitions: {},
        //                                                         },
        //                                                     },
        //                                                 ],
        //                                             },
        //                                         },
        //                                     },
        //                                 ],
        //                             },
        //                         },
        //                     },
        //                 ],
        //             },
        //         },
        //     },
        //     out_state: RefCell {
        //         value: State {
        //             accepting: true,
        //             transitions: {},
        //         },
        //     },
        // }

        let re = NFA::concat(vec![NFA::char('a'), NFA::char('b')]);
        println!("test get_transition_table re {:#?}", re);

        let table = re.get_transition_table();
        println!("test get_transition_table table {:#?}", table);
        assert_eq!(table.len(), 4);

        assert_eq!(
            table.get(&1),
            Some(&HashMap::from([
                ("ε*".to_string(), vec![1]),
                ("a".to_string(), vec![2]),
            ]))
        );

        assert_eq!(
            table.get(&2),
            Some(&HashMap::from([("ε*".to_string(), vec![2, 3])]))
        );

        assert_eq!(
            table.get(&3),
            Some(&HashMap::from([
                ("ε*".to_string(), vec![3]),
                ("b".to_string(), vec![4]),
            ]))
        );

        assert_eq!(
            table.get(&4),
            Some(&HashMap::from([("ε*".to_string(), vec![4])]))
        );
    }

    #[test]
    fn get_transition_table_or() {
        // Given regex /a|b/
        // Its graph looks like:
        //                  ε          a          ε
        //                 ---> (s:2) ---> (s:3) ---
        //                /                          \
        //  <start> -(s:1)                            - (s:4) -> <end>
        //                \                          /
        //                 ---> (s:5) ---> (s:6) ---
        //                  ε          a          ε
        //
        // Its NFA table is:
        //
        // ┌─────┬───┬───┬─────────┐
        // │     │ a │ b │ ε*      │
        // ├─────┼───┼───┼─────────┤
        // │ 1 > │   │   │ {1,2,5} │
        // ├─────┼───┼───┼─────────┤
        // │ 2   │ 3 │   │ 2       │
        // ├─────┼───┼───┼─────────┤
        // │ 3   │   │   │ {3,4}   │
        // ├─────┼───┼───┼─────────┤
        // │ 4 ✓ │   │   │ 4       │
        // ├─────┼───┼───┼─────────┤
        // │ 5   │   │ 6 │ 5       │
        // ├─────┼───┼───┼─────────┤
        // │ 6   │   │   │ {6,4}   │
        // └─────┴───┴───┴─────────┘
        //
        // And the data representation as JSON is:
        //
        // {
        //   '1': { 'ε*': [ 1, 2, 5 ] },
        //   '2': { a: [ 3 ], 'ε*': [ 2 ] },
        //   '3': { 'ε*': [ 3, 4 ] },
        //   '4': { 'ε*': [ 4 ] },
        //   '5': { b: [ 6 ], 'ε*': [ 5 ] },
        //   '6': { 'ε*': [ 6, 4 ] },
        // }
        let re = NFA::or(vec![NFA::char('a'), NFA::char('b')]);
        println!("test get_transition_table re {:#?}", re);

        let table = re.get_transition_table();
        println!("test get_transition_table table {:#?}", table);
        assert_eq!(table.len(), 6);

        assert_eq!(
            table.get(&1),
            Some(&HashMap::from([("ε*".to_string(), vec![1, 2, 5]),]))
        );

        assert_eq!(
            table.get(&2),
            Some(&HashMap::from([("ε*".to_string(), vec![2, 3])]))
        );
    }
}
