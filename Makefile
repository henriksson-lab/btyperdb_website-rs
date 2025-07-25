all:
	cargo watch -w server -w src -x "run"
	#https://github.com/sacovo/actix-yew-template

2:
	cd app; trunk watch
