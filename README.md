# Colony-Noise-Scripts
This respository contains scripts to run two-alternative choice experiments with [decide](https://github.com/melizalab/decide) framework.
For example stimuli, read [stimuli repository](https://github.com/SMMoseley/colony-noise-stimuli)

## Usage
```bash
git clone https://github.com/SMMoseley/colony-noise-stimuli
git clone https://github.com/SMMoseley/colony-noise-scripts
cd colony-noise-scripts
./generate-chorus-configs.js --stimuli-file ../colony-noise-stimuli/colony-noise-stimuli.yml --correct-choices-file ../colony-noise-stimuli/correct_choices.yml

```
The script will give an output named `chorus-config.json`. You can use this in decide to run gng.js

For example:
```
scripts/gng.js C14 @smm3rc ../colony-noise-stimuli/chorus-config.json --init-key peck_left --feed-duration 1000 --response-window 10000
```
