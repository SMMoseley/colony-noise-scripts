# Colony-Noise-Scripts
This respository contains scripts to run two-alternative choice experiments with [decide](https://github.com/melizalab/decide) framework.
For example stimuli and configuration, read [colony-noise-stimuli](https://github.com/SMMoseley/colony-noise-stimuli)

## Usage
```bash
git clone https://github.com/SMMoseley/colony-noise-stimuli
git clone https://github.com/SMMoseley/colony-noise-scripts
cd colony-noise-scripts
cargo install --patch .
cd ../colony-noise-stimuli/center-init
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
correct response. If a stimulus name matches the start of a single stimulus in
`correct_choices.yml`, then the two stimuli will be assigned the same correct response.
For example, "ztqe" and "ztqe\_30"
will be assigned correct choices based on the response given by the "ztqe" key in
`correct_choices.yml` (if either of them are present in `experiment.yml`).

By default, in order to control for the inherent properties of the stimuli,
extra configs will be created that have the opposite correct choices.
This behavior can be disabled by passing the `--no-inverted-config` flag.

## Example `experiment.yml`
```
let experiment: decide_config::Experiment = serde_yaml::from_str("
config:
  parameters: # these will be added as-is to the output config
    correct_timeout: false
    rand_replace: false
    init_position: peck_center
  output_config_name: 2ac-config # file extension will be added automatically
  stimulus_root: /root/colony-noise-stimuli/stimuli/clean_stim/
  choices: # the two alternative choices
    - peck_left
    - peck_right
stimuli: # a file will be generated for each group
  - name: 0oq8ifcb_30
    group: 2
  - name: 9ex2k0dy_30
    group: 2
  - name: c95zqjxq_30
    group: 4 # includes all smaller-numbered groups too
  - name: g29wxi4q_30
    group: 4
  - name: igmi8fxa_30
    group: 6
  - name: jkexyrd5_30
    group: 6
  - name: l1a3ltpy_30
    group: 8
  - name: p1mrfhop_30
    group: 8
  - name: vekibwgj_30
    group: 10
  - name: ztqee46x_30
    group: 10
").unwrap();
```
