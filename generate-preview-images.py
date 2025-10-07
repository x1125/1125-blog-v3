import fileinput
import os

for line in fileinput.input():
    orig_image_src = line.strip().split('(')[1].split(')')[0]
    preview_image_src = '{}/preview/{}'.format(*orig_image_src.rsplit('/', 1))
    if not os.path.isfile(preview_image_src):
        print('generating {}'.format(preview_image_src))
        os.system('convert "{}" -resize 500 "{}"'.format(orig_image_src, preview_image_src))
    pass
