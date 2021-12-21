# Colony-Noise-Scripts
This respository contains scripts to run two-alternative choice experiments with [decide](https://github.com/melizalab/decide) framework.
For example stimuli and configuration, read [colony-noise-stimuli](https://github.com/SMMoseley/colony-noise-stimuli)

## Usage
```bash
git clone https://github.com/SMMoseley/colony-noise-stimuli
git clone https://github.com/SMMoseley/colony-noise-scripts
cd colony-noise-scripts
cargo install --path .
cd ../colony-noise-stimuli/experiment
decide-config experiment.yml
```

The script will create a file named `2ac-config.json`.
You can use it as an argument for the `gng.js` script in `decide`.

For example:
```bash
scripts/gng.js C14 @smm3rc ../colony-noise-stimuli/right-init/2ac-config.json --feed-duration 1000 --response-window 10000
```

The script will also create a file named `correct_choices.yml` which maps
stimuli to the randomly assigned choices that will be rewarded. The program will
use this file so that stimuli in future config files will always have the same
correct response.

By default, in order to control for the inherent properties of the stimuli,
extra configs will be created that have the opposite correct choices.
This behavior can be disabled by passing the `--no-inverted-config` flag.

## Example `experiment.yml`
```
let experiment: decide_config::Experiment = serde_yaml::from_str("
decide:
  parameters: # these will be added as-is to the output config
    correct_timeout: false
    rand_replace: true
    init_position: peck_center
  output_config_name: 2ac-config # file extension will be added automatically
  stimulus_root: /root/colony-noise-stimuli/stimuli/snr_stim/
  choices: # the two alternative choices
    - peck_left
    - peck_right
  stimuli_subsets:
    set0: [0oq8ifcb, vekibwgj, g29wxi4q, l1a3ltpy]
    set1: [ztqee46x, jkexyrd5, mrel2o09, w08e1crn]
  include_background: true
scenes:
  padding: 2.0
  gap: 0.5
  ramp: 0.002
  foreground-dBFS: [-30]
  background-dBFS: [-20, -25, -30, -35, -40, -45, -50, -55, -60, -65, -100]
  foreground:
    - g29wxi4q
    - c95zqjxq
    - vekibwgj
    - 0oq8ifcb
    - igmi8fxa
    - p1mrfhop
    - l1a3ltpy
    - 9ex2k0dy
    - ztqee46x
    - jkexyrd5
  background:
    - h22zy3zp
").unwrap();
```
