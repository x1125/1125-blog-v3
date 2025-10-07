.PHONY: all update-dependencies compress

all: update-dependencies compress

compress:
	minifier public/assets/1125.css public/assets/modal.js

update-dependencies:
	cargo install minifier