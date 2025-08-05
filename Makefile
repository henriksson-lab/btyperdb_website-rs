1:
	cd app; trunk watch
2:
	cargo watch -w server -w src -x "run"

get_test:
	scp -rp beagle:/husky/carroll/btyperdb/minimal_testing .

