#!/usr/bin/env node

const fs = require('fs');
const YAML = require('yaml');
const _ = require('underscore');

const argv = require("yargs")
	.usage("Generate config files for chorus noise 2 alternative choice experiment")
	.describe("phase", "phase of training (1)")
	.describe("invert-answers", "whether to flip correct keys for each stimulus")
	.describe("stimuli-file", ".yml file containing a list of stimuli")
	.describe("correct-choices-file", ".yml file containing the correct choice for each stimulus")
	.describe("f", "overwrite existing files")
	.default({
		"phase": 1,
		"invert-answers": false,
	})
	.boolean("invert-answers")
	.boolean("f")
	.demand("stimuli-file")
	.argv;

const outputConfigName = 'chorus-config.json';
const alternativeChoices = ["peck_left", "peck_center"];
const wrongChoice = "peck_right";
const stimulus_root = "/root/colony-noise-stimuli/stimuli";
const parameters = {
	"correct_timeout": false,
	"rand_replace": false,
	"init_key": "peck_left",
	"feed_duration": 1000
};
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
	if (choice === alternativeChoices[0]) {
		return alternativeChoices[1];
	}
	else {
		return alternativeChoices[0];
	}
}

function getStimuli(stimuliFilename) {
	const stimuliFile = fs.readFileSync(stimuliFilename, 'utf8');
	const stimuliConfig = YAML.parse(stimuliFile);
	return stimuliConfig.stimuli;
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
		cue_resp: [cueMap[correctKey]],
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
		correctChoices = parseCorrectChoicesFile(correctChoicesFile);
	}
	if (invertAnswers) {
		correctChoices = _.mapObject(correctChoices, otherChoice);
	}
	return correctChoices;
}

function parseCorrectChoicesFile(filename) {
	const correctChoicesFile = fs.readFileSync(filename, 'utf8');
	return YAML.parse(correctChoicesFile);
}

function writeCorrectChoicesFile(correctChoices, forceWrite) {
	const output = YAML.stringify(correctChoices);
	writeFileSafe("correct_choices.yml", output, forceWrite);
}

function generateConfig(argv) {
	if (argv.phase === 1) {
		const stimuli = getStimuli(argv.stimuliFile);
		const correctChoices = getCorrectChoices(stimuli, argv.correctChoicesFile, argv.f, argv.invertAnswers);
		let config = {};
		config.parameters = parameters;
		config.stimulus_root = stimulus_root;
		config.stimuli = stimuli.map((s) => addStimuliParameters(s, correctChoices[s]));
		return config
	}
	else {
		console.error("unknown phase");
	}
}

saveConfig(outputConfigName, generateConfig(argv), argv.f);
