use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    rc::Rc,
};

use crate::nfa::NFA;
use crate::state::State;

#[derive(Debug, Clone)]
pub struct DFATable {
    pub starting_state: String,
    pub accepting_states: HashSet<String>,
    pub table: BTreeMap<String, BTreeMap<String, String>>,
}

impl DFATable {
    pub fn new() -> Self {
        DFATable {
            starting_state: "".to_string(),
            accepting_states: HashSet::new(),
            table: BTreeMap::new(),
        }
    }

    pub fn simplify_notations(&mut self) {
        let mut count = 0_usize;
        let mut hash: BTreeMap<String, String> = BTreeMap::new();
        let mut new_table = BTreeMap::new();
        let mut new_accepting_states = HashSet::new();

        for (root_states_id, transitions) in self.table.iter() {
            let new_root_states_id = hash
                .entry(root_states_id.to_string())
                .or_insert_with(|| {
                    count += 1;
                    count.to_string()
                })
                .to_owned();

            let mut new_transitions: BTreeMap<String, String> = BTreeMap::new();

            for (transition, transition_states_id) in transitions.iter() {
                let new_transition_states_id = hash
                    .entry(transition_states_id.to_string())
                    .or_insert_with(|| {
                        count += 1;
                        count.to_string()
                    })
                    .to_owned();

                new_transitions.insert(transition.to_owned(), new_transition_states_id);
            }

            new_table.insert(new_root_states_id, new_transitions);
        }

        for id in self.accepting_states.iter() {
            let new_id = hash.get(id).unwrap().to_owned();
            new_accepting_states.insert(new_id);
        }

        self.table = new_table;
        self.accepting_states = new_accepting_states;
        self.starting_state = hash.get(&self.starting_state).unwrap().to_owned();
    }

    pub fn apply_minimization(&mut self, groups: &Vec<Vec<String>>) {
        let mut new_table = BTreeMap::new();
        let mut new_accepting_states = HashSet::new();
        let mut new_labels = HashMap::new();

        for group_states in groups {
            for old_label in group_states {
                let new_label: String = group_states
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<String>>()
                    .join(",");

                new_labels.insert(old_label, new_label);
            }
        }

        for (root_states_id, transitions) in self.table.iter() {
            let new_root_states_id = new_labels.get(root_states_id).unwrap().to_owned();

            let mut new_transitions: BTreeMap<String, String> = BTreeMap::new();

            for (transition, transition_states_id) in transitions.iter() {
                let new_transition_states_id =
                    new_labels.get(&transition_states_id).unwrap().to_owned();

                new_transitions.insert(transition.to_owned(), new_transition_states_id);
            }

            new_table.insert(new_root_states_id, new_transitions);
        }

        for id in self.accepting_states.iter() {
            let new_id = new_labels.get(id).unwrap().to_owned();
            new_accepting_states.insert(new_id);
        }

        self.table = new_table;
        self.accepting_states = new_accepting_states;
        self.starting_state = new_labels.get(&self.starting_state).unwrap().to_owned();
    }

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
            starting_state: String::new(),
            accepting_states: HashSet::new(),
            table: BTreeMap::new(),
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

                let mut row: BTreeMap<String, String> = BTreeMap::new();
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
    //         Some(&BTreeMap::from([
    //             ("ε*".to_string(), vec![1]),
    //             ("a".to_string(), vec![2]),
    //         ]))
    //     );
    //
    //     assert_eq!(
    //         table.get(&2),
    //         Some(&BTreeMap::from([("ε*".to_string(), vec![2, 3])]))
    //     );
    //
    //     assert_eq!(
    //         table.get(&3),
    //         Some(&BTreeMap::from([
    //             ("ε*".to_string(), vec![3]),
    //             ("b".to_string(), vec![4]),
    //         ]))
    //     );
    //
    //     assert_eq!(
    //         table.get(&4),
    //         Some(&BTreeMap::from([("ε*".to_string(), vec![4])]))
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
        //   "1,2,5": { "a": "3,4", "b": "6,4" },
        //   "3,4": {},
        //   "6,4": {},
        // }
        let nfa = NFA::or(vec![NFA::char('a'), NFA::char('b')]);
        let mut dfa_table = DFATable::from(&nfa);
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
            Some(&BTreeMap::from([
                ("b".to_string(), "6,4".to_string()),
                ("a".to_string(), "3,4".to_string()),
            ]))
        );

        assert_eq!(
            dfa_table.table.get(&"3,4".to_string()),
            Some(&BTreeMap::new())
        );

        assert_eq!(
            dfa_table.table.get(&"6,4".to_string()),
            Some(&BTreeMap::new())
        );

        // In this second testing phase we apply the remapping to simplify the state notations.
        // The graph will look like this:
        //                  a
        //                 ---> (s:2)
        //                /
        //  <start> -(s:1)
        //                \
        //                 ---> (s:3)
        //                  b
        //
        // It should have the previous table remapped to this:
        //
        // ┌─────┬───┬───┐
        // │     │ a │ b │
        // ├─────┼───┼───┤
        // │ 1 > │ 3 │ 2 │
        // ├─────┼───┼───┤
        // │ 2 ✓ │   │   │
        // ├─────┼───┼───┤
        // │ 3 ✓ │   │   │
        // └─────┴───┴───┘
        //
        // And the data representation as JSON is:
        //
        // {
        //   "1": { "a": "2", "b": "3" },
        //   "2": {},
        //   "3": {},
        // }
        dfa_table.simplify_notations();
        println!("test get_transition_table remapped table {:#?}", dfa_table);

        assert_eq!(dfa_table.starting_state, "1".to_string());

        assert_eq!(dfa_table.accepting_states.len(), 2);
        assert_eq!(
            dfa_table.accepting_states,
            HashSet::from(["2".to_string(), "3".to_string()])
        );

        assert_eq!(dfa_table.table.len(), 3);

        assert_eq!(
            dfa_table.table.get(&"1".to_string()),
            Some(&BTreeMap::from([
                ("b".to_string(), "3".to_string()),
                ("a".to_string(), "2".to_string()),
            ]))
        );

        assert_eq!(
            dfa_table.table.get(&"2".to_string()),
            Some(&BTreeMap::new())
        );

        assert_eq!(
            dfa_table.table.get(&"3".to_string()),
            Some(&BTreeMap::new())
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
    //         Some(&BTreeMap::from([
    //             ("a".to_string(), vec![2]),
    //             ("ε*".to_string(), vec![1, 2])
    //         ]))
    //     );
    //
    //     assert_eq!(
    //         table.get(&2),
    //         Some(&BTreeMap::from([("ε*".to_string(), vec![2, 1]),]))
    //     );
    // }
}
