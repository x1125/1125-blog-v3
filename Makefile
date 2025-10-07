.PHONY: all compress-js compress-css

all: compress-js compress-css

compress-js:
	terser -o assets/script.min.js assets/script.js

compress-css:
	node-sass assets/style.css --output-style compressed > assets/style.min.css