use std::collections::{BTreeMap, HashMap, HashSet};

use crate::automata::nfa::NFA;

use super::nfa_table::NFATable;

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
        let mut dfa_table = DFATable::new();

        let epsilon_transitions_id = "ε*".to_string();

        let nfa_table = nfa.get_transition_table();
        println!("nfa_table {:#?}", nfa_table);

        let alphabet = dfa_table.get_alphabet(&nfa_table);

        let nfa_row_starting_state_transitions = nfa_table
            .table
            .get(&1)
            .expect("expected NFA table to have a state with key 1")
            .get(&epsilon_transitions_id)
            .expect("expected NFA table first row to have transition states for the epsilon closure column");

        let mut row: BTreeMap<String, String> = BTreeMap::new();
        let mut new_states_bucket: Vec<Vec<usize>> = vec![];

        for c in alphabet.iter() {
            if let Some(ids) =
                DFATable::find_epislon_closure(&c, &nfa_row_starting_state_transitions, &nfa_table)
            {
                let states = ids
                    .iter()
                    .map(|&n| n.to_string())
                    .collect::<Vec<String>>()
                    .join(",");

                println!("found for char {c} new state={:?}", ids);

                row.insert(c.to_owned(), states);
                new_states_bucket.push(ids);
            }
        }

        let start_state_label = nfa_row_starting_state_transitions
            .iter()
            .map(|&n| n.to_string())
            .collect::<Vec<String>>()
            .join(",");

        dfa_table.starting_state = start_state_label.to_owned();
        dfa_table.table.insert(start_state_label, row);
        println!("nfa_accepting {:?}", nfa_table.accepting_states);
        println!("new_states_bucket {:?}", new_states_bucket);

        // Now that we found the starting state, we need to resolve any new states found during the states lookup.
        while new_states_bucket.len() > 0 {
            let new_states = new_states_bucket.pop().unwrap();
            let mut row: BTreeMap<String, String> = BTreeMap::new();

            let label = new_states
                .iter()
                .map(|&n| n.to_string())
                .collect::<Vec<String>>()
                .join(",");

            let accepting = new_states
                .iter()
                .any(|&s| nfa_table.accepting_states.contains(&s));

            println!("new_states {:?}", new_states);

            if accepting {
                dfa_table.accepting_states.insert(label.to_owned());
                dfa_table.table.insert(label, row);
                continue;
            }

            for c in alphabet.iter() {
                if let Some(ids) = DFATable::find_epislon_closure(&c, &new_states, &nfa_table) {
                    let states = ids
                        .iter()
                        .map(|&n| n.to_string())
                        .collect::<Vec<String>>()
                        .join(",");

                    if !dfa_table.table.contains_key(&states) {
                        new_states_bucket.push(ids);
                    }

                    row.insert(c.to_owned(), states);
                }
            }

            dfa_table.table.insert(label, row);
        }

        println!("dfa_table {:#?}", dfa_table);
        dfa_table
    }

    fn find_epislon_closure(
        c: &String,
        states: &Vec<usize>,
        nfa_table: &NFATable,
    ) -> Option<Vec<usize>> {
        let epsilon_transitions_id = "ε*".to_string();
        let mut states = states.clone();
        let mut active = false;

        println!("find_epislon_closure char={}", c);
        while states.len() > 0 {
            println!("  states={:?}", states);
            let state_id = states.pop().unwrap();
            println!("  state_id={}", state_id);

            if let Some(row) = nfa_table.table.get(&state_id) {
                println!("    row={:?}", row);

                if let Some(ids) = row.get(c) {
                    active = true;
                    println!("      deep lookup ids={:?}", ids);
                    states.extend(ids);
                } else if active {
                    if let Some(ids) = row.get(&epsilon_transitions_id) {
                        println!("      found={:?}", ids);
                        return Some(ids.to_vec());
                    }
                }
            }
        }

        None
    }

    // DFA alphabet. Same as in NFA, except ε.
    fn get_alphabet(&self, nfa_table: &NFATable) -> HashSet<String> {
        nfa_table
            .table
            .values()
            .map(|map| map.keys().cloned().collect::<Vec<String>>())
            .flatten()
            .filter(|c| *c != "ε*".to_string())
            .collect()
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
    fn get_transition_table_single_nfa() {
        // Given regex /a/
        // Its NFA graph looks like:
        //                 ε          a
        //  <start> (s:1) ---> (s:2) ---> <end>
        //
        // Its NFA table is:
        //
        // ┌─────┬───┬────┐
        // │     │ a │ ε* │
        // ├─────┼───┼────┤
        // │ 1 > │ 2 │ 1  │
        // ├─────┼───┼────┤
        // │ 2 ✓ │   │ 2  │
        // └─────┴───┴────┘
        //
        // Its DFA table is:
        //
        // ┌─────┬───┐
        // │     │ a │
        // ├─────┼───┤
        // │ 1 > │ 2 │
        // ├─────┼───┤
        // │ 2 ✓ │   │
        // └─────┴───┘
        //
        // And the data representation as JSON is:
        //
        // {
        //   "1": { "a": "2" },
        //   "2": {},
        // }
        let nfa = NFA::char('a');
        let dfa_table = DFATable::from(&nfa);
        // println!("test get_transition_table table {:#?}", dfa_table);

        assert_eq!(dfa_table.starting_state, "1".to_string());
        assert_eq!(dfa_table.accepting_states, HashSet::from(["2".to_string()]));
        assert_eq!(dfa_table.table.len(), 2);

        assert_eq!(
            dfa_table.table.get(&"1".to_string()),
            Some(&BTreeMap::from([("a".to_string(), "2".to_string()),]))
        );

        assert_eq!(
            dfa_table.table.get(&"2".to_string()),
            Some(&BTreeMap::new())
        );
    }

    #[test]
    fn get_transition_table_single_or() {
        // Given regex /a|b/.
        //
        // Its NFA graph looks like:
        //                  ε          a          ε
        //                 ---> (s:2) ---> (s:3) ---
        //                /                          \
        //  <start> -(s:1)                            - (s:4) -> <end>
        //                \                          /
        //                 ---> (s:5) ---> (s:6) ---
        //                  ε          a          ε
        //
        // And the NFA table is:
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
        let nfa = NFA::or(vec![NFA::char('a'), NFA::char('b')]);
        let mut dfa_table = DFATable::from(&nfa);
        // println!("test get_transition_table table {:#?}", dfa_table);

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
        dfa_table.simplify_notations();
        // println!("test get_transition_table remapped table {:#?}", dfa_table);

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

    #[test]
    fn get_transition_table_or_and_concat() {
        // Given regex /a|bc/.
        //
        //
        // And the NFA table is:
        //
        // ┌─────┬───┬───┬───┬─────────┐
        // │     │ a │ b │ c │ ε*      │
        // ├─────┼───┼───┼───┼─────────┤
        // │ 1 > │   │   │   │ {1,2,5} │
        // ├─────┼───┼───┼───┼─────────┤
        // │ 2   │ 3 │   │   │ 2       │
        // ├─────┼───┼───┼───┼─────────┤
        // │ 3   │   │   │   │ {3,4}   │
        // ├─────┼───┼───┼───┼─────────┤
        // │ 4 ✓ │   │   │   │ 4       │
        // ├─────┼───┼───┼───┼─────────┤
        // │ 5   │   │ 6 │   │ 5       │
        // ├─────┼───┼───┼───┼─────────┤
        // │ 6   │   │   │   │ {6,7}   │
        // ├─────┼───┼───┼───┼─────────┤
        // │ 7   │   │   │ 8 │ 7       │
        // ├─────┼───┼───┼───┼─────────┤
        // │ 8   │   │   │   │ {8,4}   │
        // └─────┴───┴───┴───┴─────────┘
        //
        // Its DFA table is:
        //
        //
        // This is how the DFA table is built from the NFA table:
        //
        // 1,2,5 starting       | a: 2->3->3,4 | b: 5->6->6,7 | c:
        // 6,7                  | a:           | b:           | c: 7->8->8,4
        // 8,4   accepting (bc) | b:           | b:           | c:
        // 3,4   accepting (a)  | a:           | b:           | c:
        //
        let nfa = NFA::or(vec![
            NFA::char('a'),
            NFA::concat(vec![NFA::char('b'), NFA::char('c')]),
        ]);
        let mut dfa_table = DFATable::from(&nfa);
        // println!("test get_transition_table table {:#?}", dfa_table);

        assert_eq!(dfa_table.starting_state, "1,2,5".to_string());

        assert_eq!(dfa_table.accepting_states.len(), 2);
        assert_eq!(
            dfa_table.accepting_states,
            HashSet::from(["3,4".to_string(), "8,4".to_string()])
        );

        assert_eq!(dfa_table.table.len(), 4);

        assert_eq!(
            dfa_table.table.get(&"1,2,5".to_string()),
            Some(&BTreeMap::from([
                ("b".to_string(), "6,7".to_string()),
                ("a".to_string(), "3,4".to_string()),
            ]))
        );

        assert_eq!(
            dfa_table.table.get(&"3,4".to_string()),
            Some(&BTreeMap::new())
        );

        assert_eq!(
            dfa_table.table.get(&"6,7".to_string()),
            Some(&BTreeMap::from([("c".to_string(), "8,4".to_string())]))
        );

        assert_eq!(
            dfa_table.table.get(&"8,4".to_string()),
            Some(&BTreeMap::new())
        );

        // In this second testing phase we apply the remapping to simplify the state notations.
        //
        dfa_table.simplify_notations();
        println!("test get_transition_table remapped table {:#?}", dfa_table);

        assert_eq!(dfa_table.starting_state, "1".to_string());

        assert_eq!(dfa_table.accepting_states.len(), 2);
        assert_eq!(
            dfa_table.accepting_states,
            HashSet::from(["2".to_string(), "4".to_string()])
        );

        assert_eq!(dfa_table.table.len(), 4);

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
            Some(&BTreeMap::from([("c".to_string(), "4".to_string()),]))
        );

        assert_eq!(
            dfa_table.table.get(&"4".to_string()),
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
