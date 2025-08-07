1:
        mkdir -p app/assets
	cd app; trunk watch
2:
	cargo watch -w server -w src -x "run"

get_test:
	scp -rp beagle:/husky/carroll/btyperdb/minimal_testing .


build:
	mkdir -p app/assets
	cd app; trunk build
	cargo build

serve: build
	cargo run
