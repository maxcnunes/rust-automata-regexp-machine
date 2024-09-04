var util = require('node:util');
var State = /** @class */ (function () {
    function State() {
        this.accepting = false;
        this.transitionMap = new Map();
    }
    State.prototype.addTransitionForSymbol = function (symbol, state) {
        var t = this.transitionMap.get(symbol);
        if (t == null) {
            t = [];
        }
        t.push(state);
        this.transitionMap.set(symbol, t);
    };
    State.prototype.getTransitionsForSymbol = function (symbol) {
        return this.transitionMap.get(symbol) || [];
    };
    return State;
}());
var NFA = /** @class */ (function () {
    function NFA(inState, outState) {
        this.inState = inState;
        this.outState = outState;
    }
    return NFA;
}());
function char(symbol) {
    var inState = new State();
    var outState = new State();
    outState.accepting = true;
    inState.addTransitionForSymbol(symbol, outState);
    return new NFA(inState, outState);
}
var EPSILON = 'ε';
function epsilon() {
    return char(EPSILON);
}
function concatPair(first, second) {
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
function concat(first) {
    var rest = [];
    for (var _i = 1; _i < arguments.length; _i++) {
        rest[_i - 1] = arguments[_i];
    }
    for (var _a = 0, rest_1 = rest; _a < rest_1.length; _a++) {
        var fragment = rest_1[_a];
        first = concatPair(first, fragment);
    }
    return first;
}
debugger;
var re = concat(char('a'), char('b'));
console.log(':::: final NFA:', util.inspect(re, { compact: false, depth: 20, breakLength: 200, colors: true }));
