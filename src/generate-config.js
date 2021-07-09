#!/usr/bin/env node

const io = require('./io.js');
const Choice = require('./choice.js').Choice;
const ChoiceCode = require('./choice.js').ChoiceCode;
const constants = require('./constants.js');

const argv = require("yargs")
	.usage("Generate config files for chorus noise 2 alternative choice experiment")
	.option("experiment-file", {
		describe: "yaml file containing a list of stimuli and parameters",
		demandOption: true,
		normalize: true
	})
	.option("correct-choices-file", {
		describe: "yaml file containing the correct choice for each stimulus. New file will be generated if not provided",
		normalize: true,
	})
	.option("phase", {
		describe: "phase of training (phase 1 includes cue lights)",
		type: 'number',
		default: 1
	})
	.option("invert-answers", {
		describe: "whether to flip correct keys for each stimulus",
		type: 'boolean'
	})
	.option("f", {
		alias: 'force-write',
		describe: "overwrite existing files",
		type: 'boolean'
	})
	.argv;

const experimentConfig = io.parseYamlFile(argv.experimentFile);
const outputConfig = generateOutputConfig(
	experimentConfig,
	argv.correctChoicesFile,
	argv.invertAnswers,
	argv.phase,
	argv.f
);
io.saveObject(experimentConfig.config['output_config_name'], outputConfig, argv.f);

function generateOutputConfig(experimentConfig, correctChoicesFile, invertAnswers, phase, forceWrite) {
	const stimuli = experimentConfig.stimuli;
	const choiceMap = new ChoiceCode(experimentConfig.config.choices);
	const correctChoices = getCorrectChoices(stimuli, correctChoicesFile, choiceMap, forceWrite);
	if (invertAnswers) {
		correctChouices = invertChoices(correctChoices);
	}
	let config = {};
	config.parameters = experimentConfig.config.parameters;
	config['stimulus_root'] = experimentConfig.config['stimulus_root'];
	config.stimuli = stimuli.map((s) => addStimuliParameters(
		experimentConfig.config,
		s,
		correctChoices[s],
		phase
	));
	return config;
}

function getCorrectChoices(stimuli, correctChoicesFile, choiceMap, forceWrite, invertAnswers) {
	let decodedCorrectChoices;
	if (correctChoicesFile === undefined) {
		correctChoices = Choice.assignChoicesToKeys(stimuli);
		decodedCorrectChoices = choiceMap.decodeValues(correctChoices);
		io.writeCorrectChoicesFile(decodedCorrectChoices, forceWrite);
	}
	else {
		decodedCorrectChoices = io.parseYamlFile(correctChoicesFile);
	}
	return decodedCorrectChoices;
}

function invertChoices(decodedChoices) {
		const encodedChoices = choiceMap.encodeValues(decodedChoices);
		invertedChoices = Choice.invertValues(encodedChoices);
		return choiceMap.decodeValues(correctChoices);
}

function addStimuliParameters(experimentConfig, stimuliName, correctKey, phase) {
	let responses = {};
	const alternativeChoices = experimentConfig.choices;
	for (const key of experimentConfig.keys) {
		let r;
		if (key === correctKey) {
			r = constants.correctResponse;
		}
		else if (alternativeChoices.indexOf(key) != -1) {
			r = constants.incorrectResponse;
		}
		else {
			r = constants.incorrectResponse;
		}
		responses[key] = r;
	};
	responses["timeout"] = constants.neutralResponse;
	let stimuliConfig = {
		name: stimuliName,
		frequency: 1,
		responses: responses,
	};
	if (phase <= 1 && correctKey) {
		stimuliConfig['cue_resp'] = [constants.cueMap[correctKey]];
	}
	return stimuliConfig;
}
