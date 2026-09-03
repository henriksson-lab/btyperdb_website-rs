.PHONY: build serve ingest watch-app watch-server get_test

# Release is the default: the wasm bundle is downloaded by every visitor, and
# a debug build of it is roughly 5x larger.
build:
	mkdir -p app/assets
	cd app && trunk build --release
	cargo build --release

serve: build
	cargo run --release

# Rebuild meta/data.sqlite from meta/btyperdb.tsv. Pass the directory holding
# btyperdb.tsv, e.g.  make ingest STORE=/husky/vignesh/BTyperDB/latest/meta
ingest:
	cargo run --release -p ingest -- $(STORE) --replace

# Development: debug builds that rebuild on change.
watch-app:
	mkdir -p app/assets
	cd app && trunk watch

watch-server:
	cargo watch -w server -w src -x "run"

get_test:
	scp -rp beagle:/husky/carroll/btyperdb/minimal_testing .
