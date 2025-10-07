.PHONY: all compress-js compress-css

all: compress-js compress-css

compress-js:
	terser -o assets/script.min.js assets/script.js

compress-css:
	node-sass assets/style.css --output-style compressed > assets/style.min.css

prepare-images: lowercase-filenames exif-scrub

lowercase-filenames:
	find posts/ -depth -name '*.*' -type f -exec bash -c 'base=${0%.*} ext=${0##*.} a=$base.${ext,,}; [ "$a" != "$0" ] && mv -- "$0" "$a"' {} \;

exif-scrub:
	find posts/ -iname '*.jpg' | xargs exiftool -all=
	find posts/ -iname '*.jpg_original' | xargs -r rm