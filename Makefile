.PHONY: all compress-js compress-css

all: compress-js compress-css prepare-images generate-preview-images check-references

compress-js:
	terser -o assets/script.min.js assets/script.js
	terser -o assets/markdown-it-attrs.min.js assets/markdown-it-attrs.js
	terser -o assets/markdown-it-footnote-bulma.min.js assets/markdown-it-footnote-bulma.js
	terser -o assets/markdown-it-tags.min.js assets/markdown-it-tags.js
	terser -o assets/OrbitControls.min.js assets/OrbitControls.js
	terser -o assets/STLLoader.min.js assets/STLLoader.js

compress-css:
	node-sass assets/style.css --output-style compressed > assets/style.min.css

prepare-images: lowercase-filenames exif-scrub

lowercase-filenames:
	find posts/ -depth -name '*.*' -type f -exec bash -c 'base=${0%.*} ext=${0##*.} a=$base.${ext,,}; [ "$a" != "$0" ] && mv -- "$0" "$a"' {} \;

exif-scrub:
	find posts/ -iname '*.jpg' | xargs exiftool -all=
	find posts/ -iname '*.jpg_original' | xargs -r rm

generate-bulma-from-scss:
	npm run build

generate-preview-images:
	. venv/bin/activate; grep -r '{.previewimage}' posts/ | python3 generate-preview-images.py

check-references:
	. venv/bin/activate; find posts/ -name '*.md' | python3 check-references.py