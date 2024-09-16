use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::nfa::NFA;
use crate::state::State;

#[derive(Debug, Clone)]
pub struct DFATable {
    state_count: usize,
    map_state_ids: HashMap<*const State, usize>,
    visited: HashSet<*const State>,

    pub starting_state: String,
    pub accepting_states: HashSet<String>,
    pub table: HashMap<String, HashMap<String, String>>,
}

impl DFATable {
    pub fn from(nfa: &NFA) -> Self {
        // Epsilon closure (denotaed as "ε*"): set of states reacheable from this state,
        // following only ε-transitions. Which means, it contains the state of it self and
        // the states it is transition to.
        //
        // Example table structure for the NFA with regexp /a|b/:
        // {
        //   '1': { 'ε*': [ 1, 2, 5 ] },
        //   '2': { a: [ 3 ], 'ε*': [ 2 ] },
        //   '3': { 'ε*': [ 3, 4 ] },
        //   '4': { 'ε*': [ 4 ] },
        //   '5': { b: [ 6 ], 'ε*': [ 5 ] },
        //   '6': { 'ε*': [ 6, 4 ] },
        // }
        //
        // It will be converted to DFA table like this:
        //
        // {
        //   '1,2,5': { 'a': '3,4', 'b': '6,4' },
        //   '3,4': {},
        //   '6,4': {},
        // }
        let mut builder = DFATable {
            state_count: 0,
            starting_state: String::new(),
            accepting_states: HashSet::new(),
            table: HashMap::new(),
            map_state_ids: HashMap::new(),
            visited: HashSet::new(),
        };

        let epsilon_transitions_id = "ε*".to_string();

        let nfa_table = nfa.get_transition_table();
        println!("nfa_table {:#?}", nfa_table);

        for state_id in 1..=nfa_table.table.len() {
            let transitions = nfa_table.table.get(&state_id).unwrap();
            println!("state_id {:?} transitions {:?}", state_id, transitions);
            if transitions.len() == 1 && transitions.get(&epsilon_transitions_id).is_some() {
                let transition_states = transitions.get(&epsilon_transitions_id).unwrap();

                // There is only the epsilon transition, we don't record it into the DFA table in this case.
                if transition_states.len() == 1 {
                    continue;
                }

                let label: String = transition_states
                    .iter()
                    .map(|&n| n.to_string())
                    .collect::<Vec<String>>()
                    .join(",");

                let mut row: HashMap<String, String> = HashMap::new();
                println!("  label {:?}", label);

                // Skip the first state, since it is the ε source state.
                // Example: loop over [1, 2, 5] from 'ε*', skipping the first state.
                for state_id in transition_states.iter().skip(1) {
                    println!("    state_id {:?}", state_id);
                    // Example: first itration, get { a: [ 3 ], 'ε*': [ 2 ] } for state 2.
                    let child_transitions = nfa_table.table.get(&state_id).unwrap();
                    println!("    child_transitions {:?}", child_transitions);

                    for (child_state_id, child_states) in child_transitions {
                        println!(
                            "      child_state_id {:?} child_states {:?}",
                            child_state_id, child_states
                        );
                        if *child_state_id != epsilon_transitions_id {
                            // Example: loop over [ 3 ] for the "a" transiction.
                            for inner_state_id in child_states {
                                println!("        inner_state_id {:?}", inner_state_id);
                                // Example: get { 'ε*': [ 3, 4 ] } for state 3.
                                let transitions = nfa_table.table.get(&inner_state_id).unwrap();
                                println!("        transitions {:?}", transitions);
                                if transitions.len() == 1
                                    && transitions.get(&epsilon_transitions_id).is_some()
                                {
                                    // Example: get [ 3, 4 ] for transition 'ε*'.
                                    let transition_states =
                                        transitions.get(&epsilon_transitions_id).unwrap();

                                    let transition_states_label = transition_states
                                        .iter()
                                        .map(|&n| n.to_string())
                                        .collect::<Vec<String>>()
                                        .join(",");
                                    println!(
                                        "        transition_states_label {:?}",
                                        transition_states_label
                                    );

                                    // Example: insert { "a": "3,4" } to "1,2,5" label row.
                                    row.insert(
                                        child_state_id.to_string(),
                                        transition_states_label.to_owned(),
                                    );

                                    let accepting = transition_states
                                        .iter()
                                        .any(|&ts| nfa_table.accepting_states.contains(&ts));

                                    if accepting {
                                        builder.accepting_states.insert(transition_states_label);
                                    }

                                    break;
                                }
                            }
                        }
                    }
                }

                if builder.table.len() == 0 {
                    builder.starting_state = label.to_owned();
                }

                builder.table.insert(label, row.to_owned());
            }
        }

        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn get_transition_table_concat() {
    //     // Given regex /ab/
    //     // Its graph looks like:
    //     //                    a          ε          b
    //     //  (s:1 - starting) ---> (s:2) ---> (s:3) ---> (s:4 - accepting)
    //     //
    //     // Its NFA table is:
    //     //
    //     // ┌─────┬───┬───┬───────┐
    //     // │     │ a │ b │ ε*    │
    //     // ├─────┼───┼───┼───────┤
    //     // │ 1 > │ 2 │   │ 1     │
    //     // ├─────┼───┼───┼───────┤
    //     // │ 2   │   │   │ {2,3} │
    //     // ├─────┼───┼───┼───────┤
    //     // │ 3   │   │ 4 │ 3     │
    //     // ├─────┼───┼───┼───────┤
    //     // │ 4 ✓ │   │   │ 4     │
    //     // └─────┴───┴───┴───────┘
    //     //
    //     // And the data representation as JSON is:
    //     //
    //     // {
    //     //   '1': { a: [ 2 ], 'ε*': [ 1 ] },
    //     //   '2': { 'ε*': [ 2, 3 ] },
    //     //   '3': { b: [ 4 ], 'ε*': [ 3 ] },
    //     //   '4': { 'ε*': [ 4 ] },
    //     // }
    //     //  {
    //     //     in_state: RefCell {                                                          -> state:1 - starting
    //     //         value: State {
    //     //             accepting: false,
    //     //             transitions: {
    //     //                 'a': [                                                           -> transition:a
    //     //                     RefCell {
    //     //                         value: State {                                           -> state:2
    //     //                             accepting: false,
    //     //                             transitions: {
    //     //                                 'ε': [                                           -> transition:ε
    //     //                                     RefCell {
    //     //                                         value: State {                           -> state:3
    //     //                                             accepting: false,
    //     //                                             transitions: {
    //     //                                                 'b': [                           -> transition:b
    //     //                                                     RefCell {
    //     //                                                         value: State {           -> state:4 - accepting
    //     //                                                             accepting: true,
    //     //                                                             transitions: {},
    //     //                                                         },
    //     //                                                     },
    //     //                                                 ],
    //     //                                             },
    //     //                                         },
    //     //                                     },
    //     //                                 ],
    //     //                             },
    //     //                         },
    //     //                     },
    //     //                 ],
    //     //             },
    //     //         },
    //     //     },
    //     //     out_state: RefCell {
    //     //         value: State {
    //     //             accepting: true,
    //     //             transitions: {},
    //     //         },
    //     //     },
    //     // }
    //
    //     let re = NFA::concat(vec![NFA::char('a'), NFA::char('b')]);
    //     println!("test get_transition_table re {:#?}", re);
    //
    //     let table = re.get_transition_table();
    //     println!("test get_transition_table table {:#?}", table);
    //     assert_eq!(table.len(), 4);
    //
    //     assert_eq!(
    //         table.get(&1),
    //         Some(&HashMap::from([
    //             ("ε*".to_string(), vec![1]),
    //             ("a".to_string(), vec![2]),
    //         ]))
    //     );
    //
    //     assert_eq!(
    //         table.get(&2),
    //         Some(&HashMap::from([("ε*".to_string(), vec![2, 3])]))
    //     );
    //
    //     assert_eq!(
    //         table.get(&3),
    //         Some(&HashMap::from([
    //             ("ε*".to_string(), vec![3]),
    //             ("b".to_string(), vec![4]),
    //         ]))
    //     );
    //
    //     assert_eq!(
    //         table.get(&4),
    //         Some(&HashMap::from([("ε*".to_string(), vec![4])]))
    //     );
    // }

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
        // Its DFA table is:
        //
        // ┌─────────┬─────┬─────┐
        // │         │ a   │ b   │
        // ├─────────┼─────┼─────┤
        // │ 1,2,5 > │ 3,4 │ 6,4 │
        // ├─────────┼─────┼─────┤
        // │ 3,4 ✓   │     │     │
        // ├─────────┼─────┼─────┤
        // │ 6,4 ✓   │     │     │
        // └─────────┴─────┴─────┘
        //
        // And the data representation as JSON is:
        //
        // {
        //   "1,2,5": { "a": "3,4", "ε*": "6,4" },
        //   "3,4": {},
        //   "6,4": {},
        // }
        let nfa = NFA::or(vec![NFA::char('a'), NFA::char('b')]);
        let dfa_table = DFATable::from(&nfa);
        println!("test get_transition_table table {:#?}", dfa_table);

        assert_eq!(dfa_table.starting_state, "1,2,5".to_string());

        assert_eq!(dfa_table.accepting_states.len(), 2);
        assert_eq!(
            dfa_table.accepting_states,
            HashSet::from(["3,4".to_string(), "6,4".to_string()])
        );

        assert_eq!(dfa_table.table.len(), 3);

        assert_eq!(
            dfa_table.table.get(&"1,2,5".to_string()),
            Some(&HashMap::from([
                ("b".to_string(), "6,4".to_string()),
                ("a".to_string(), "3,4".to_string()),
            ]))
        );

        assert_eq!(
            dfa_table.table.get(&"3,4".to_string()),
            Some(&HashMap::new())
        );

        assert_eq!(
            dfa_table.table.get(&"6,4".to_string()),
            Some(&HashMap::new())
        );
    }

    // #[test]
    // fn get_transition_table_rep() {
    //     // Given regex /a*/
    //     //
    //     // The original zero or more NFA graph would look like this:
    //     //                        .------------------.
    //     //                       \/                  |
    //     //                 ε          a          ε
    //     //  <start> (s:1) ---> (s:2) ---> (s:3) ---> (s:4) <end>
    //     //                                           /\
    //     //                |                          |
    //     //                .-------------------------.
    //     //                             ε
    //     //
    //     // But we are using an optimized implementation,
    //     // so the graph looks like this instead:
    //     //
    //     //            .-----------.
    //     //           \/           |
    //     //                 a
    //     //  <start> (s:1) --->  (s:4) <end>
    //     //                       /\
    //     //           |           |
    //     //           .-----------.
    //     //                 ε
    //     //
    //     // Its NFA table is:
    //     //
    //     // ┌─────┬───┬───────┐
    //     // │     │ a │ ε*    │
    //     // ├─────┼───┼───────┤
    //     // │ 1 > │ 2 │ {1,2} │
    //     // ├─────┼───┼───────┤
    //     // │ 2 ✓ │   │ {2,1} │
    //     // └─────┴───┴───────┘
    //     //
    //     // And the data representation as JSON is:
    //     //
    //     // {
    //     //   '1': { a: [ 2 ], 'ε*': [ 1,2 ] },
    //     //   '2': { 'ε*': [ 2, 1 ] },
    //     // }
    //     let re = NFA::rep(NFA::char('a'));
    //
    //     let table = re.get_transition_table();
    //     println!("test get_transition_table table {:#?}", table);
    //     assert_eq!(table.len(), 2);
    //
    //     assert_eq!(
    //         table.get(&1),
    //         Some(&HashMap::from([
    //             ("a".to_string(), vec![2]),
    //             ("ε*".to_string(), vec![1, 2])
    //         ]))
    //     );
    //
    //     assert_eq!(
    //         table.get(&2),
    //         Some(&HashMap::from([("ε*".to_string(), vec![2, 1]),]))
    //     );
    // }
}
