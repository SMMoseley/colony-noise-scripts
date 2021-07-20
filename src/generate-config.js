#!/usr/bin/env node

const _ = require('underscore');
const Fraction = require('fraction.js');

const io = require('./io.js');
const Choice = require('./choice.js').Choice;
const ChoiceCode = require('./choice.js').ChoiceCode;
const constants = require('./constants.js');

const argv = require("yargs")
	.usage("Generate config files for chorus noise 2 alternative choice experiment")
	.option("experiment-file", {
		alias: 'e',
		describe: "yaml file containing a list of stimuli and parameters",
		demandOption: true,
		normalize: true
	})
	.option("correct-choices-file", {
		alias: 'c',
		describe: "yaml file containing the correct choice for each stimulus. New file will be generated if not provided",
		normalize: true,
	})
	.option("phase", {
		alias: 'p',
		describe: "phase of training (phase 1 includes cue lights)",
		type: 'number',
		default: 1
	})
	.option("invert-answers", {
		alias: 'i',
		describe: "whether to flip correct keys for each stimulus",
		type: 'boolean'
	})
	.option("force-write", {
		alias: 'f',
		describe: "overwrite existing files",
		type: 'boolean'
	})
	.completion()
	.argv;

const experimentConfig = io.parseYamlFile(argv.experimentFile);
const outputConfig = generateOutputConfig(
	experimentConfig,
	argv.correctChoicesFile,
	argv.invertAnswers,
	argv.phase,
	argv.forceWrite
);
const outputConfigName = io.makeConfigName(
	experimentConfig.config['output_config_name'],
	argv.invertAnswers,
	argv.phase
);
io.saveObject(outputConfigName, outputConfig, argv.forceWrite);

function generateOutputConfig(experimentConfig, correctChoicesFile, invertAnswers, phase, forceWrite) {
	const stimuli = experimentConfig.stimuli;
	const choiceMap = new ChoiceCode(experimentConfig.config.choices);
	let correctChoices = getCorrectChoices(stimuli, correctChoicesFile, choiceMap, forceWrite);
	if (invertAnswers) {
		correctChoices = invertChoices(correctChoices, choiceMap);
	}
	let config = {};
	config.parameters = experimentConfig.config.parameters;
	config['stimulus_root'] = experimentConfig.config['stimulus_root'];
	config.stimuli = generateStimuliList(
		stimuli,
		correctChoices,
		experimentConfig.config.choices,
		experimentConfig.config.keys,
		phase
	);
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

function invertChoices(decodedChoices, choiceMap) {
		const encodedChoices = choiceMap.encodeValues(decodedChoices);
		invertedChoices = Choice.invertValues(encodedChoices);
		return choiceMap.decodeValues(invertedChoices);
}

function generateStimuliList(stimuli, correctChoices, choices, allKeys, phase) {
	return _.flatten(stimuli.map((s) => {
		const phase1_stim = addStimuliParameters(
			s,
			choices,
			allKeys,
			correctChoices[s],
			true,
		);
		const phase2_stim = addStimuliParameters(
			s,
			choices,
			allKeys,
			correctChoices[s],
			false,
		);
		if (phase <= 1) {
			return phase1_stim;
		}
		else if (phase < 2) {
			const frac = new Fraction(phase - 1);
			phase2_stim.frequency = frac.n;
			phase1_stim.frequency = frac.d - frac.n;
			return [
				phase1_stim,
				phase2_stim
			]
		}
		else {
			return phase2_stim;
		}
	}));
}

function addStimuliParameters(stimulusName, choices, allKeys, correctKey, cueLights) {
	let responses = {};
	for (const key of allKeys) {
		let r;
		if (key === correctKey) {
			r = constants.correctResponse;
		}
		else if (choices.indexOf(key) != -1) {
			r = constants.incorrectResponse;
		}
		else {
			r = constants.incorrectResponse;
		}
		responses[key] = r;
	};
	responses["timeout"] = constants.neutralResponse;
	let stimuliConfig = {
		name: stimulusName,
		frequency: 1,
		responses: responses,
		category: "no_cue_lights",
	};
	if (cueLights && correctKey) {
		stimuliConfig['cue_resp'] = [constants.cueMap[correctKey]];
		stimuliConfig.category = "cue_lights";
	}
	return stimuliConfig;
}
