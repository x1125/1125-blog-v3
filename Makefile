.PHONY: all update-dependencies compress-css generate-bulma-from-scss

all: update-dependencies compress-css generate-bulma-from-scss

compress-css: generate-bulma-from-scss
	css-minifier -l 2 -i public/assets/bulmaswatch.custom.css -o public/assets/bulmaswatch.custom.min.css
	css-minifier -l 2 -i public/assets/style.css -o public/assets/style.min.css

generate-bulma-from-scss:
	sass-rs < public/assets/scss/1125-bulma.scss > public/assets/bulmaswatch.custom.css

update-dependencies:
	cargo install sass-rs css-minifier