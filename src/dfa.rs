use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet},
    rc::Rc,
};

use crate::{dfa_table::DFATable, nfa::NFA};

#[derive(Debug, Clone)]
pub struct DFA {
    table: DFATable,
}

impl DFA {
    pub fn from(nfa: &NFA) -> Self {
        DFA {
            table: DFATable::from(nfa),
        }
    }

    // DFA alphabet. Same as in NFA, except ε.
    pub fn get_alphabet() -> HashSet<String> {
        todo!()
    }

    pub fn get_starting_state(&self) -> String {
        self.table.starting_state.to_owned()
    }

    // Accepting states (calculated during table built).
    pub fn get_accepting_states(&self) -> HashSet<String> {
        self.table.accepting_states.to_owned()
    }

    // DFA transition table built from NFA table.
    pub fn get_transition_table(&self) -> BTreeMap<String, BTreeMap<String, String>> {
        self.table.table.to_owned()
    }

    pub fn simplify_notations(&mut self) {
        self.table.simplify_notations();
    }

    // Minimize this DFA.
    // To check whether they are equivalent we should see where we go from each character from these states,
    // if we go to the states which belong to the same group, the original states are equivalent.
    // Even if the actual states are different, the important is whether the outgoing state belongs to the same group.
    pub fn minimize(&mut self) -> bool {
        let mut groups = vec![];
        let mut minimizing = true;
        let mut minimized = false;

        while minimizing {
            minimizing = self.minimize_once(&mut groups);
            if minimizing {
                minimized = true;
            }
        }

        if minimized {
            self.table.apply_minimization(&groups);
        }

        minimized
    }

    fn minimize_once(&mut self, groups: &mut Vec<Vec<String>>) -> bool {
        let groups_len = groups.len();
        if groups_len == 0 {
            let mut non_accepting_states = Vec::new();
            let mut accepting_states = Vec::new();

            for (state, _) in self.table.table.iter() {
                if self.table.accepting_states.contains(state) {
                    accepting_states.push(state.to_owned());
                } else {
                    non_accepting_states.push(state.to_owned());
                }
            }

            groups.push(non_accepting_states);
            groups.push(accepting_states);

            return true;
        }

        let mut base_group_iter = groups[0].to_owned().into_iter();
        let mut state_a = base_group_iter.next().unwrap();
        let mut new_base_group = vec![state_a.to_owned()];

        while let Some(state_b) = base_group_iter.next() {
            println!("-----------------");
            println!("state_a {:?}", state_a);
            println!("state_b {:?}", state_b);

            let state_a_transitions = self.table.table.get(&state_a).unwrap();
            let state_b_transitions = self.table.table.get(&state_b).unwrap();
            println!("state_a_transitions {:#?}", state_a_transitions);
            println!("state_b_transitions {:#?}", state_b_transitions);

            let mut moved = false;

            for (transition, transition_state_a) in state_a_transitions {
                let transition_state_b = state_b_transitions.get(transition).unwrap();

                let group = &groups[0];
                println!("group {:#?}", group);
                println!(
                    "contains transition_state_a {:?} {:#?}",
                    transition_state_a,
                    group.contains(transition_state_a)
                );
                println!(
                    "contains transition_state_b {:?} {:#?}",
                    transition_state_b,
                    group.contains(transition_state_b)
                );
                if transition_state_a != transition_state_b
                    && (!group.contains(transition_state_a) || !group.contains(transition_state_b))
                {
                    groups.push(vec![state_b.to_owned()]);
                    moved = true;
                }
            }

            if !moved {
                new_base_group.push(state_b.to_owned());
                state_a = state_b;
            }
        }
        println!("END-----------------");

        if groups[0].len() != new_base_group.len() {
            groups[0] = new_base_group;
            println!("groups {:#?}", groups);
            return true;
        }

        println!("groups {:#?}", groups);
        false
    }

    // Tests whether this DFA accepts the string.
    fn test(&self, string: &String) -> bool {
        let table = self.get_transition_table();
        println!("test table={:#?}", &table);

        let mut state = self.get_starting_state();
        println!("test starting_state={:#?}", &state);

        for c in string.chars() {
            println!("test c={c} state={:?}", state);
            let looking_state = table.get(&state).unwrap().get(&c.to_string());

            if let Some(s) = looking_state {
                state = s.to_string();
            } else {
                return false;
            }
        }

        self.get_accepting_states().contains(&state)
    }
}

#[cfg(test)]
mod tests {
    use crate::dfa::DFA;

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
    fn test_or() {
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
        //   "1,2,5": { a: "3,4", 'ε*': "6,4" },
        //   "3,4": {},
        //   "6,4": {},
        // }
        let nfa = NFA::or(vec![NFA::char('a'), NFA::char('b')]);
        let re = DFA::from(&nfa);

        assert_eq!(re.test(&"a".to_string()), true);
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

    #[test]
    fn minimize_once_table() {
        // Given a DFA graph like this:
        //
        //                           a           a
        //                          ___     __________
        //                         |   \   /         |
        //                         \   |  |          |
        //                    a     \ \/ \/   b      |
        //                 ------> ( s:2 ) -----> ( s:4 )
        //                /         /\ /\            |
        //               /          |  |    a        |
        //  <start> -(s:1)        a |  |________     | b
        //               \          |           \    |
        //                \         |            \  \/
        //                 ------> ( s:3 ) <---- ( s:5 ) -> <end>
        //                    b    /  /\     b
        //                        |   |
        //                        \__/
        //                          b
        //
        // Its DFA table is:
        //
        // ┌─────┬───┬───┐
        // │     │ a │ b │
        // ├─────┼───┼───┤
        // │ 1 > │ 2 │ 3 │
        // ├─────┼───┼───┤
        // │ 2   │ 2 │ 4 │
        // ├─────┼───┼───┤
        // │ 3   │ 2 │ 3 │
        // ├─────┼───┼───┤
        // │ 4   │ 2 │ 5 │
        // ├─────┼───┼───┤
        // │ 5 ✓ │ 2 │ 3 │
        // └─────┴───┴───┘
        //
        let mut dfa_table = DFATable::new();
        dfa_table.starting_state = "1".to_string();
        dfa_table.accepting_states = HashSet::from([("5".to_string())]);
        dfa_table.table = BTreeMap::from([
            (
                "1".to_string(),
                BTreeMap::from([
                    ("a".to_string(), "2".to_string()),
                    ("b".to_string(), "3".to_string()),
                ]),
            ),
            (
                "2".to_string(),
                BTreeMap::from([
                    ("a".to_string(), "2".to_string()),
                    ("b".to_string(), "4".to_string()),
                ]),
            ),
            (
                "3".to_string(),
                BTreeMap::from([
                    ("a".to_string(), "2".to_string()),
                    ("b".to_string(), "3".to_string()),
                ]),
            ),
            (
                "4".to_string(),
                BTreeMap::from([
                    ("a".to_string(), "2".to_string()),
                    ("b".to_string(), "5".to_string()),
                ]),
            ),
            (
                "5".to_string(),
                BTreeMap::from([
                    ("a".to_string(), "2".to_string()),
                    ("b".to_string(), "3".to_string()),
                ]),
            ),
        ]);
        println!("test minimize_table table {:#?}", dfa_table);

        let mut dfa = DFA {
            table: dfa_table.to_owned(),
        };

        let mut groups = vec![];

        // First step: 0-equivalence
        println!("------ First step");
        let minimized = dfa.minimize_once(&mut groups);
        assert_eq!(minimized, true);
        assert_eq!(groups.len(), 2);

        assert_eq!(
            groups[0],
            [
                "1".to_string(),
                "2".to_string(),
                "3".to_string(),
                "4".to_string()
            ]
        );

        assert_eq!(groups[1], ["5".to_string()]);

        // Second step: 1-equivalence
        println!("------ Second step");
        let minimized = dfa.minimize_once(&mut groups);
        assert_eq!(minimized, true);
        assert_eq!(groups.len(), 3);

        assert_eq!(
            groups[0],
            ["1".to_string(), "2".to_string(), "3".to_string()]
        );

        assert_eq!(groups[1], ["5".to_string()]);
        assert_eq!(groups[2], ["4".to_string()]);

        // Third step: 2-equivalence
        println!("------ Third step");
        let minimized = dfa.minimize_once(&mut groups);
        assert_eq!(minimized, true);
        assert_eq!(groups.len(), 4);

        assert_eq!(groups[0], ["1".to_string(), "3".to_string()]);

        assert_eq!(groups[1], ["5".to_string()]);
        assert_eq!(groups[2], ["4".to_string()]);
        assert_eq!(groups[3], ["2".to_string()]);

        // Forth step: 3-equivalence
        println!("------ Forth step");
        let minimized = dfa.minimize_once(&mut groups);
        assert_eq!(minimized, false);
        assert_eq!(groups.len(), 4);

        assert_eq!(groups[0], ["1".to_string(), "3".to_string()]);
        assert_eq!(groups[1], ["5".to_string()]);
        assert_eq!(groups[2], ["4".to_string()]);
        assert_eq!(groups[3], ["2".to_string()]);
    }

    #[test]
    fn minimize_table() {
        // Given a DFA graph like this:
        //
        //                           a           a
        //                          ___     __________
        //                         |   \   /         |
        //                         \   |  |          |
        //                    a     \ \/ \/   b      |
        //                 ------> ( s:2 ) -----> ( s:4 )
        //                /         /\ /\            |
        //               /          |  |    a        |
        //  <start> -(s:1)        a |  |________     | b
        //               \          |           \    |
        //                \         |            \  \/
        //                 ------> ( s:3 ) <---- ( s:5 ) -> <end>
        //                    b    /  /\     b
        //                        |   |
        //                        \__/
        //                          b
        //
        // Its DFA table is:
        //
        // ┌─────┬───┬───┐
        // │     │ a │ b │
        // ├─────┼───┼───┤
        // │ 1 > │ 2 │ 3 │
        // ├─────┼───┼───┤
        // │ 2   │ 2 │ 4 │
        // ├─────┼───┼───┤
        // │ 3   │ 2 │ 3 │
        // ├─────┼───┼───┤
        // │ 4   │ 2 │ 5 │
        // ├─────┼───┼───┤
        // │ 5 ✓ │ 2 │ 3 │
        // └─────┴───┴───┘
        //
        let mut dfa_table = DFATable::new();
        dfa_table.starting_state = "1".to_string();
        dfa_table.accepting_states = HashSet::from([("5".to_string())]);
        dfa_table.table = BTreeMap::from([
            (
                "1".to_string(),
                BTreeMap::from([
                    ("a".to_string(), "2".to_string()),
                    ("b".to_string(), "3".to_string()),
                ]),
            ),
            (
                "2".to_string(),
                BTreeMap::from([
                    ("a".to_string(), "2".to_string()),
                    ("b".to_string(), "4".to_string()),
                ]),
            ),
            (
                "3".to_string(),
                BTreeMap::from([
                    ("a".to_string(), "2".to_string()),
                    ("b".to_string(), "3".to_string()),
                ]),
            ),
            (
                "4".to_string(),
                BTreeMap::from([
                    ("a".to_string(), "2".to_string()),
                    ("b".to_string(), "5".to_string()),
                ]),
            ),
            (
                "5".to_string(),
                BTreeMap::from([
                    ("a".to_string(), "2".to_string()),
                    ("b".to_string(), "3".to_string()),
                ]),
            ),
        ]);
        println!("test minimize_table table {:#?}", dfa_table);

        let mut dfa = DFA {
            table: dfa_table.to_owned(),
        };

        let minimized = dfa.minimize();
        assert_eq!(minimized, true);
        assert_eq!(
            dfa.table.table,
            BTreeMap::from([
                (
                    "1,3".to_string(),
                    BTreeMap::from([
                        ("a".to_string(), "2".to_string()),
                        ("b".to_string(), "1,3".to_string()),
                    ]),
                ),
                (
                    "2".to_string(),
                    BTreeMap::from([
                        ("a".to_string(), "2".to_string()),
                        ("b".to_string(), "4".to_string()),
                    ]),
                ),
                (
                    "4".to_string(),
                    BTreeMap::from([
                        ("a".to_string(), "2".to_string()),
                        ("b".to_string(), "5".to_string()),
                    ]),
                ),
                (
                    "5".to_string(),
                    BTreeMap::from([
                        ("a".to_string(), "2".to_string()),
                        ("b".to_string(), "1,3".to_string()),
                    ]),
                ),
            ])
        );

        dfa.simplify_notations();
        assert_eq!(
            dfa.table.table,
            BTreeMap::from([
                (
                    "1".to_string(),
                    BTreeMap::from([
                        ("a".to_string(), "2".to_string()),
                        ("b".to_string(), "1".to_string()),
                    ]),
                ),
                (
                    "2".to_string(),
                    BTreeMap::from([
                        ("a".to_string(), "2".to_string()),
                        ("b".to_string(), "3".to_string()),
                    ]),
                ),
                (
                    "3".to_string(),
                    BTreeMap::from([
                        ("a".to_string(), "2".to_string()),
                        ("b".to_string(), "4".to_string()),
                    ]),
                ),
                (
                    "4".to_string(),
                    BTreeMap::from([
                        ("a".to_string(), "2".to_string()),
                        ("b".to_string(), "1".to_string()),
                    ]),
                ),
            ])
        );
    }
}
