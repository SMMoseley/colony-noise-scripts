#!/usr/bin/env node

const fs = require('fs');
const YAML = require('yaml');
const _ = require('underscore');

const argv = require("yargs")
	.usage("Generate config files for chorus noise 2 alternative choice experiment")
	.describe("phase", "phase of training (phase 1 includes cue lights)")
	.number("phase")
	.describe("invert-answers", "whether to flip correct keys for each stimulus")
	.describe("experiment-file", ".yml file containing a list of stimuli and parameters")
	.describe("correct-choices-file", ".yml file containing the correct choice for each stimulus")
	.describe("f", "overwrite existing files")
	.default({
		"phase": 1,
		"invert-answers": false,
	})
	.boolean("invert-answers")
	.boolean("f")
	.demand("experiment-file")
	.argv;

const codedChoices = {
	A: "A",
	B: "B"
};
const alternativeChoices = ["peck_left", "peck_left"];
const wrongChoice = "peck_right";
const cueMap = {
  "peck_left": "left_blue",
  "peck_right": "right_blue",
  "peck_center": "center_blue",
};
const correctResponse = {
	p_reward: 1.0,
	correct: true,
};
const incorrectResponse = {
	p_punish: 1.0,
	correct: false,
};
const neutralResponse = {
	correct: false,
};

function otherChoice(choice) {
	if (choice === codedChoices.A) {
		return codedChoices.B;
	}
	else {
		return codedChoices.A;
	}
}

function saveConfig(filename, data, forceWrite) {
	writeFileSafe(filename, JSON.stringify(data, null, "  "), forceWrite);
}

function writeFileSafe(filename, data, forceWrite) {
	if (fs.existsSync(filename) && !forceWrite) {
		throw new Error(filename + " already exists");
	}
	else {
		fs.writeFileSync(filename, data);
	}
}

function addStimuliParameters(stimuliName, correctKey, phase) {
	let responses = {};
	for (const key of alternativeChoices) {
		if (key === correctKey) {
			responses[key] = correctResponse;
		}
		else {
			responses[key] =  incorrectResponse;
		}
	};
	responses[wrongChoice] = incorrectResponse;
	responses["timeout"] = neutralResponse;
	let stimuliConfig = {
		name: stimuliName,
		frequency: 1,
		responses: responses,
	};
	if (phase <= 1) {
		stimuliConfig.cue_resp = [cueMap[correctKey]];
	}
	return stimuliConfig;
}

function assignCorrectChoices(stimuli) {
	let correctChoices = {}
	for (const s of stimuli) {
		correctChoices[s] = codedChoices.A;
	};
	for (const s of _.sample(stimuli, stimuli.length/2)) {
		correctChoices[s] = codedChoices.B;
	};
	return correctChoices;
}

function getCorrectChoices(stimuli, correctChoicesFile, choiceMap, forceWrite, invertAnswers) {
	let decodedCorrectChoices;
	if (correctChoicesFile === undefined) {
		correctChoices = assignCorrectChoices(stimuli);
		decodedCorrectChoices = decodeChoices(correctChoices, choiceMap);
		writeCorrectChoicesFile(decodedCorrectChoices, forceWrite);
	}
	else {
		decodedCorrectChoices = parseYamlFile(correctChoicesFile);
	}
	if (invertAnswers) {
		const encodedCorrectChoices = encodeChoices(decodedCorrectChoices, choiceMap);
		correctChoices = _.mapObject(encodedCorrectChoices, otherChoice);
		decodedCorrectChoices = decodeChoices(correctChoices, choiceMap);
	}
	return decodedCorrectChoices;
}

function decodeChoices(choices, choiceMap) {
	return _.mapObject(choices, (c) => choiceMap.decode(c));
}

function encodeChoices(choices, choiceMap) {
	return _.mapObject(choices, (c) => choiceMap.encode(c));
}

function parseYamlFile(filename) {
	const file = fs.readFileSync(filename, 'utf8');
	return YAML.parse(file);
}

function writeCorrectChoicesFile(correctChoices, forceWrite) {
	const output = YAML.stringify(correctChoices);
	writeFileSafe("correct_choices.yml", output, forceWrite);
}

function makeChoiceMap(choicesList) {
	choiceMap = {};
	choiceMap[codedChoices.A] = choicesList[0];
	choiceMap[codedChoices.B] = choicesList[1];
	backwardsChoiceMap = {};
	backwardsChoiceMap[choicesList[0]] = codedChoices.A;
	backwardsChoiceMap[choicesList[1]] = codedChoices.B;
	return {
		decode(choice) {
			return choiceMap[choice];
		},
		encode(key) {
			return backwardsChoiceMap[key];
		}
	}
}

function generateOutputConfig(experimentConfig, correctChoicesFile, invertAnswers, phase, forceWrite) {
	const stimuli = experimentConfig.stimuli;
	const choiceMap = makeChoiceMap(experimentConfig.config.choices);
	const correctChoices = getCorrectChoices(stimuli, correctChoicesFile, choiceMap, forceWrite, invertAnswers);
	let config = {};
	config.parameters = experimentConfig.config.parameters;
	config.stimulus_root = experimentConfig.config.stimulus_root;
	config.stimuli = stimuli.map((s) => addStimuliParameters(s, correctChoices[s], phase));
	return config;
}

const experimentConfig = parseYamlFile(argv.experimentFile);
const outputConfig = generateOutputConfig(
	experimentConfig,
	argv.correctChoicesFile,
	argv.invertAnswers,
	argv.phase,
	argv.f
);
saveConfig(experimentConfig.config.output_config_name, outputConfig, argv.f);
