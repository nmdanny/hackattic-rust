const fs = require('fs');
const {promisify} = require('util');

const symbols = ['+','-','×','÷'];

const open = promisify(fs.open);
const appendFile = promisify(fs.appendFile)

const args = process.argv.slice(2);
const lines = args[0] || 8 * 125;

async function create_training_text(filename, withSymbols) {
	let file = await open(filename, 'w');
	for (let i = 0; i < lines; i++) {
		let op = symbols[Math.floor(Math.random() * symbols.length)];
		if (!withSymbols)
			op = '';
		const string = op + Math.random().toString(10).substr(2,7) + '\n';
		await appendFile(file, string); 
	}
}

create_training_text('training_text_no_ops.txt',false);
create_training_text('training_text_with_ops.txt',true);



function getRandomInt(min, max) {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}