const fs = require('fs');
const YAML = require('yaml');

module.exports = class IO {
	static saveObject(filename, data, forceWrite) {
		IO.writeFileSafe(filename, JSON.stringify(data, null, "  "), forceWrite);
	}

	static writeFileSafe(filename, data, forceWrite) {
		if (fs.existsSync(filename) && !forceWrite) {
			throw new Error(`${filename} already exists`);
		}
		else {
			fs.writeFileSync(filename, data);
		}
	}

	static parseYamlFile(filename) {
		const file = fs.readFileSync(filename, 'utf8');
		return YAML.parse(file);
	}

	static writeCorrectChoicesFile(correctChoices, forceWrite) {
		const output = YAML.stringify(correctChoices);
		IO.writeFileSafe("correct_choices.yml", output, forceWrite);
	}

	static makeConfigName(name, invertAnswers, phase) {
		if (invertAnswers) {
			name += "-inverted";
		}
		return `${name}-p${phase}.json`;
	}
}
