const util = require('node:util');

class State {
  accepting: boolean;
  transitionMap: Map<string, Array<State>>;

  constructor() {
    this.accepting = false;
    this.transitionMap = new Map<string, Array<State>>();
  }

  addTransitionForSymbol(symbol: string, state: State): void {
    let t = this.transitionMap.get(symbol);
    if (t == null) {
      t = [];
    }
    t.push(state);
    this.transitionMap.set(symbol, t);
  }

  getTransitionsForSymbol(symbol: string): Array<State> {
    return this.transitionMap.get(symbol) || [];
  }
}

class NFA {
  inState: State;
  outState: State;

  constructor(inState: State, outState: State) {
    this.inState = inState;
    this.outState = outState;
  }
}

function char(symbol: string): NFA {
  const inState = new State();
  const outState = new State();

  outState.accepting = true;

  inState.addTransitionForSymbol(symbol, outState);

  return new NFA(inState, outState);
}

const EPSILON = 'ε';

function epsilon(): NFA {
  return char(EPSILON);
}

function concatPair(first: NFA, second: NFA): NFA {
  first.outState.accepting = false;
  second.outState.accepting = true;

console.log(":::: concat: updated_fragment before ::::");
console.log(":::: concat: updated_fragment: before first", util.inspect(first, { compact: false, depth: 20, breakLength: 200, colors: true }));
console.log(":::: concat: updated_fragment: before second", util.inspect(second, { compact: false, depth: 20, breakLength: 200, colors: true }));
  first.outState.addTransitionForSymbol(EPSILON, second.inState);
console.log(":::: concat: updated_fragment after ::::");
console.log(":::: concat: updated_fragment: after first", util.inspect(first, { compact: false, depth: 20, breakLength: 200, colors: true }));
console.log(":::: concat: updated_fragment: after second", util.inspect(second, { compact: false, depth: 20, breakLength: 200, colors: true }));

  return new NFA(first.inState, second.outState);
}

function concat(first: NFA, ...rest: Array<NFA>): NFA {
  for (let fragment of rest) {
    first = concatPair(first, fragment);
  }
  return first;
}

debugger;
const re = concat(char('a'), char('b'));
console.log(':::: final NFA:', util.inspect(re, { compact: false, depth: 20, breakLength: 200, colors: true }));
