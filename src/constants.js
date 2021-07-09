module.exports = {
	cueMap: {
		"peck_left": "left_blue",
		"peck_right": "right_blue",
		"peck_center": "center_blue",
	},
	correctResponse: {
		p_reward: 1.0,
		correct: true,
	},
	incorrectResponse: {
		p_punish: 1.0,
		correct: false,
	},
	neutralResponse: {
		correct: false,
	},
}
