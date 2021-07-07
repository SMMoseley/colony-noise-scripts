#!/usr/bin/env node

const fs = require('fs');
const YAML = require('yaml');
const _ = require('underscore');

const argv = require("yargs")
	.usage("Generate config files for chorus noise 2 alternative choice experiment")
	.describe("phase", "phase of training (1)")
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

const alternativeChoices = ["peck_left", "peck_left"];
const wrongChoice = "peck_right";
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
	if (choice === alternativeChoices[0]) {
		return alternativeChoices[1];
	}
	else {
		return alternativeChoices[0];
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

function addStimuliParameters(stimuliName, correctKey) {
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
	return {
		name: stimuliName,
		frequency: 1,
		responses: responses,
	};
}

function assignCorrectChoices(stimuli) {
	let correctChoices = {}
	for (const s of stimuli) {
		correctChoices[s] = alternativeChoices[0];
	};
	for (const s of _.sample(stimuli, stimuli.length/2)) {
		correctChoices[s] = alternativeChoices[1];
	};
	return correctChoices;
}

function getCorrectChoices(stimuli, correctChoicesFile, forceWrite, invertAnswers) {
	let correctChoices;
	if (correctChoicesFile === undefined) {
		correctChoices = assignCorrectChoices(stimuli);
		writeCorrectChoicesFile(correctChoices, forceWrite);
	}
	else {
		correctChoices = parseYamlFile(correctChoicesFile);
	}
	if (invertAnswers) {
		correctChoices = _.mapObject(correctChoices, otherChoice);
	}
	return correctChoices;
}

function parseYamlFile(filename) {
	const file = fs.readFileSync(filename, 'utf8');
	return YAML.parse(file);
}

function writeCorrectChoicesFile(correctChoices, forceWrite) {
	const output = YAML.stringify(correctChoices);
	writeFileSafe("correct_choices.yml", output, forceWrite);
}

function generateOutputConfig(experimentConfig, correctChoicesFile, invertAnswers, phase, f) {
	if (phase === 1) {
		const stimuli = experimentConfig.stimuli;
		const correctChoices = getCorrectChoices(stimuli, correctChoicesFile, f, invertAnswers);
		let config = {};
		config.parameters = experimentConfig.config.parameters;
		config.stimulus_root = experimentConfig.config.stimulus_root;
		config.stimuli = stimuli.map((s) => addStimuliParameters(s, correctChoices[s]));
		return config
	}
	else {
		console.error("unknown phase");
	}
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
