mod dfa;
mod dfa_table;
mod nfa;
mod nfa_table;
mod state;

use nfa::NFA;

fn main() {
    println!("Regex automata course!");
    let re = NFA::concat(vec![NFA::char('a'), NFA::char('b'), NFA::char('c')]);
    println!(":::: final NFA={:#?}", re);
}
