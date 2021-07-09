const _ = require('underscore');

class Choice {
	static A = new Choice("A");
	static B = new Choice("B");

	constructor(code) {
		this.code = code;
	}

	invert() {
		if (_.isEqual(this, Choice.A)) {
			return Choice.B;
		}
		else {
			return Choice.A;
		}
	}

	static invertValues(choices) {
		return _.mapObject(choices, (c) => c.invert());
	}

	static assignChoicesToKeys = (keys) => {
		let correctChoices = {}
		// Set all keys to choice A
		for (const k of keys) {
			correctChoices[k] = Choice.A;
		};
		// Then randomly sample half to switch to choice B
		const half = Math.floor(keys.length/2)
		for (const k of _.sample(keys, half)) {
			correctChoices[k] = Choice.B;
		};
		return correctChoices;
	}
}

exports.ChoiceCode = class ChoiceCode { 
	constructor(choicesList) {
		this.decodeMap = {};
		this.decodeMap[Choice.A.code] = choicesList[0];
		this.decodeMap[Choice.B.code] = choicesList[1];
		this.encodeMap = {};
		this.encodeMap[choicesList[0]] = Choice.A;
		this.encodeMap[choicesList[1]] = Choice.B;
	}

	decode(choice) {
		return this.decodeMap[choice.code];
	}

	encode(key) {
		return this.encodeMap[key];
	}

	// { foo: "A" } -> { foo: "peck_left"}
	decodeValues(choices) {
		return _.mapObject(choices, (c) => this.decode(c));
	}

	encodeValues(choices) {
		return _.mapObject(choices, (c) => this.encode(c));
	}
}

exports.Choice = Choice;
