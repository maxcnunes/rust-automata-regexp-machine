use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::nfa::NFA;
use crate::state::State;

#[derive(Debug)]
pub struct NFATable {
    state_count: usize,
    map_state_ids: HashMap<*const State, usize>,
    visited: HashSet<*const State>,

    pub starting_state: usize,
    pub accepting_states: HashSet<usize>,
    pub table: HashMap<usize, HashMap<String, Vec<usize>>>,
}

impl NFATable {
    pub fn from(nfa: &NFA) -> Self {
        // Epsilon closure (denotaed as "ε*"): set of states reacheable from this state,
        // following only ε-transitions. Which means, it contains the state of it self and
        // the states it is transition to.
        //
        // Example table structure for the NFA with regexp /ab/:
        // {
        //   '1': { a: [ 2 ], 'ε*': [ 1 ] },
        //   '2': { 'ε*': [ 2, 3 ] },
        //   '3': { b: [ 4 ], 'ε*': [ 3 ] },
        //   '4': { 'ε*': [ 4 ] },
        // }
        let mut builder = NFATable {
            state_count: 0,
            map_state_ids: HashMap::new(),
            starting_state: 0,
            accepting_states: HashSet::new(),
            visited: HashSet::new(),
            table: HashMap::new(),
        };

        builder.walk_state(nfa.in_state.to_owned());

        builder
    }

    fn walk_state(&mut self, ref_state: Rc<RefCell<State>>) {
        let state = &*ref_state.borrow();
        let state_id = self.get_state_id(state);
        // println!(
        //     "walk_state state_id={} table_len={}",
        //     state_id,
        //     self.table.len()
        // );

        if self.starting_state == 0 {
            self.starting_state = state_id;
        }

        let ptr = state as *const State;
        if self.visited.contains(&ptr) {
            return;
        }
        self.visited.insert(ptr.to_owned());

        let mut row: HashMap<String, Vec<usize>> = HashMap::new();
        row.insert("ε*".to_string(), vec![state_id]);

        for (t, states) in &state.transitions {
            let transition_label = match t {
                'ε' => "ε*".to_string(),
                c => c.to_string(),
            };

            let ids = row.entry(transition_label).or_insert(vec![]);

            for child_state in states {
                let child_state_id = self.get_state_id(&child_state.borrow());
                ids.push(child_state_id);
                self.walk_state(child_state.to_owned());
            }
        }

        if state.accepting {
            self.accepting_states.insert(state_id);
        }

        self.table.insert(state_id, row.to_owned());
    }

    fn get_state_id(&mut self, state: &State) -> usize {
        let ptr = state as *const State;

        let state_id = self.map_state_ids.entry(ptr).or_insert_with(|| {
            self.state_count += 1;
            self.state_count
        });

        *state_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let nfa_table = re.get_transition_table();
        println!("test get_transition_table table {:#?}", nfa_table);
        assert_eq!(nfa_table.starting_state, 1);

        assert_eq!(nfa_table.accepting_states.len(), 1);
        assert_eq!(nfa_table.accepting_states, HashSet::from([4]));

        assert_eq!(nfa_table.table.len(), 4);

        assert_eq!(
            nfa_table.table.get(&1),
            Some(&HashMap::from([
                ("ε*".to_string(), vec![1]),
                ("a".to_string(), vec![2]),
            ]))
        );

        assert_eq!(
            nfa_table.table.get(&2),
            Some(&HashMap::from([("ε*".to_string(), vec![2, 3])]))
        );

        assert_eq!(
            nfa_table.table.get(&3),
            Some(&HashMap::from([
                ("ε*".to_string(), vec![3]),
                ("b".to_string(), vec![4]),
            ]))
        );

        assert_eq!(
            nfa_table.table.get(&4),
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

        let nfa_table = re.get_transition_table();
        println!("test get_transition_table table {:#?}", nfa_table);
        assert_eq!(nfa_table.starting_state, 1);

        assert_eq!(nfa_table.accepting_states.len(), 1);
        assert_eq!(nfa_table.accepting_states, HashSet::from([4]));

        assert_eq!(nfa_table.table.len(), 6);

        assert_eq!(
            nfa_table.table.get(&1),
            Some(&HashMap::from([("ε*".to_string(), vec![1, 2, 5]),]))
        );

        assert_eq!(
            nfa_table.table.get(&2),
            Some(&HashMap::from([
                ("a".to_string(), vec![3]),
                ("ε*".to_string(), vec![2])
            ]))
        );

        assert_eq!(
            nfa_table.table.get(&3),
            Some(&HashMap::from([("ε*".to_string(), vec![3, 4]),]))
        );

        assert_eq!(
            nfa_table.table.get(&4),
            Some(&HashMap::from([("ε*".to_string(), vec![4])]))
        );

        assert_eq!(
            nfa_table.table.get(&5),
            Some(&HashMap::from([
                ("b".to_string(), vec![6]),
                ("ε*".to_string(), vec![5])
            ]))
        );

        assert_eq!(
            nfa_table.table.get(&6),
            Some(&HashMap::from([("ε*".to_string(), vec![6, 4])]))
        );
    }

    #[test]
    fn get_transition_table_rep() {
        // Given regex /a*/
        //
        // The original zero or more NFA graph would look like this:
        //                        .------------------.
        //                       \/                  |
        //                 ε          a          ε
        //  <start> (s:1) ---> (s:2) ---> (s:3) ---> (s:4) <end>
        //                                           /\
        //                |                          |
        //                .-------------------------.
        //                             ε
        //
        // But we are using an optimized implementation,
        // so the graph looks like this instead:
        //
        //            .-----------.
        //           \/           |
        //                 a
        //  <start> (s:1) --->  (s:4) <end>
        //                       /\
        //           |           |
        //           .-----------.
        //                 ε
        //
        // Its NFA table is:
        //
        // ┌─────┬───┬───────┐
        // │     │ a │ ε*    │
        // ├─────┼───┼───────┤
        // │ 1 > │ 2 │ {1,2} │
        // ├─────┼───┼───────┤
        // │ 2 ✓ │   │ {2,1} │
        // └─────┴───┴───────┘
        //
        // And the data representation as JSON is:
        //
        // {
        //   '1': { a: [ 2 ], 'ε*': [ 1,2 ] },
        //   '2': { 'ε*': [ 2, 1 ] },
        // }
        let re = NFA::rep(NFA::char('a'));

        let nfa_table = re.get_transition_table();
        println!("test get_transition_table table {:#?}", nfa_table);
        assert_eq!(nfa_table.starting_state, 1);

        assert_eq!(nfa_table.accepting_states.len(), 1);
        assert_eq!(nfa_table.accepting_states, HashSet::from([2]));

        assert_eq!(nfa_table.table.len(), 2);

        assert_eq!(
            nfa_table.table.get(&1),
            Some(&HashMap::from([
                ("a".to_string(), vec![2]),
                ("ε*".to_string(), vec![1, 2])
            ]))
        );

        assert_eq!(
            nfa_table.table.get(&2),
            Some(&HashMap::from([("ε*".to_string(), vec![2, 1]),]))
        );
    }
}
