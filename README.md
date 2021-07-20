# Colony-Noise-Scripts
This respository contains scripts to run two-alternative choice experiments with [decide](https://github.com/melizalab/decide) framework.
For example stimuli and configuration, read [colony-noise-stimuli](https://github.com/SMMoseley/colony-noise-stimuli)

## Usage
```bash
git clone https://github.com/SMMoseley/colony-noise-stimuli
git clone https://github.com/SMMoseley/colony-noise-scripts
cd colony-noise-scripts
npm link
cd ../colony-noise-stimuli/right-init
decide-config --experiment-file experiment.yml --phase 2
```

The script will create a file named `2ac-config-p2.json`.
You can use it as an argument for the `gng.js` script in `decide`.

For example:
```bash
scripts/gng.js C14 @smm3rc ../colony-noise-stimuli/right-init/2ac-config-inverted-p2.json --feed-duration 1000 --response-window 10000
```

The script will also create a file named `correct_choices.yml` which maps
stimuli to the randomly assigned choices that will be rewarded. You can
create additional decide config files with the same assignments by passing
in the `--correct-choices-file` flag, pointed to that file. If you would
like to control for the inherent properties of the stimuli, you can
create configs that have the opposite correct choices by additonally
passing the boolean `--invert-answers` flag.
