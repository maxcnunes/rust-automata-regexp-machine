mod ast;
mod automata;
mod error;
mod regex;

use automata::{dfa::DFA, nfa_table::NFATable, state};
use std::{cmp::Ordering, collections::HashSet};

use clap::{Parser, Subcommand};
use term_table::{row::Row, table_cell::*, Table, TableStyle};

use crate::regex::Regex;

/// Automata RegExp machine
#[derive(Parser, Debug)]
#[clap(version)]
struct Args {
    /// RegExp
    #[arg(short, long)]
    regexp: String,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Test a regex
    ///
    /// This command will open your $EDITOR, wait for you
    /// to write something, and then save the file to your
    /// garden
    Test {
        /// Input to test
        #[clap(short, long)]
        input: String,
    },

    Table {
        /// Simplify notations
        #[arg(short, long)]
        simplify_notations: bool,
    },
}

fn main() {
    println!("Regex automata course!");
    let args = Args::parse();

    let r = Regex::new(&args.regexp).unwrap();

    match args.cmd {
        Commands::Test { input } => {
            assert_eq!(r.test(&input), true);
        }
        Commands::Table { simplify_notations } => {
            let nfa = r.nfa.clone();
            let mut dfa = r.dfa.clone();
            // println!(":::: final NFA={:#?}", re);
            let nfa_table = nfa.get_transition_table();
            // println!(":::: nfa table ={:#?}", nfa_table);

            // println!(":::: dfa table={:#?}", dfa_table);
            if simplify_notations {
                dfa.simplify_notations();
            }

            println!("");
            println!("> - starting");
            println!("✓ - accepting");
            println!("");

            println!("NFA: Transition table:");
            println!("");
            print_nfa_table(&nfa_table);

            println!("DFA: Original transition table:");
            println!("");
            print_dfa_table(&dfa);

            dfa.minimize();
            if simplify_notations {
                dfa.simplify_notations();
            }
            println!("DFA: Minimized transition table");
            println!("");
            print_dfa_table(&dfa);
        }
    };
}

fn print_nfa_table(nfa_table: &NFATable) {
    let mut table_states = nfa_table.table.keys().collect::<Vec<_>>();
    table_states.sort();

    let mut map_table_transitions = HashSet::new();
    let mut table_transitions = vec![];

    for (_, transitions) in &nfa_table.table {
        for (t, _) in transitions {
            if !map_table_transitions.contains(t) {
                map_table_transitions.insert(t.to_owned());
                table_transitions.push(t);
            }
        }
    }

    table_transitions.sort_by(|&a, &b| {
        if a == b {
            return Ordering::Equal;
        }

        if *a == state::EPSILON_TRANSITIONS {
            return Ordering::Greater;
        }

        if *b == state::EPSILON_TRANSITIONS {
            return Ordering::Less;
        }

        a.cmp(b)
    });

    let mut header = Row::empty();
    header.add_cell(TableCell::new(""));
    for t in &table_transitions {
        header.add_cell(TableCell::new(t.to_string()));
    }

    let mut table = Table::builder().style(TableStyle::thin()).build();

    table.add_row(header);

    for state in table_states {
        let transitions = nfa_table.table.get(&state).unwrap();
        let mut row = Row::empty();
        let mut label = state.to_string();
        if nfa_table.starting_state == *state {
            label = format!("> {label}");
        } else if nfa_table.accepting_states.contains(state) {
            label = format!("✓ {label}");
        }

        row.add_cell(TableCell::new(label));

        for transition in &table_transitions {
            if let Some(transition_states) = transitions.get(*transition) {
                let v: String = transition_states
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<String>>()
                    .join(",");
                row.add_cell(TableCell::new(v.to_string()));
            } else {
                row.add_cell(TableCell::new("".to_string()));
            }
        }
        table.add_row(row);
    }

    println!("{}", table.render());
}

fn print_dfa_table(dfa: &DFA) {
    let dfa_table = dfa.get_transition_table();
    let table_states = dfa_table.keys().collect::<Vec<_>>();

    let first_state_row = dfa_table.get(table_states[0]).unwrap();
    let table_transitions = first_state_row.keys().collect::<Vec<_>>();

    let mut header = Row::empty();
    header.add_cell(TableCell::new(""));
    for t in table_transitions {
        header.add_cell(TableCell::new(t.to_string()));
    }

    let mut table = Table::builder().style(TableStyle::thin()).build();

    table.add_row(header);

    for (state, transitions) in dfa_table {
        let mut row = Row::empty();
        let mut label = state.to_string();
        if dfa.get_starting_state() == *state {
            label = format!("> {label}");
        } else if dfa.get_accepting_states().contains(&state) {
            label = format!("✓ {label}");
        }
        row.add_cell(TableCell::new(label));

        for (_transition, transition_states) in transitions {
            row.add_cell(TableCell::new(transition_states.to_string()));
        }
        table.add_row(row);
    }

    println!("{}", table.render());
}
