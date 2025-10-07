import os, json

posts_path = 'posts'

if posts_path[len(posts_path) - 1] != '/':
    posts_path += '/'


def generate_index(path):
    index = set()
    with os.scandir(path) as it:
        for entry in it:
            if entry.is_dir():
                sub_index = generate_index('{}{}/'.format(path, entry.name))
                if len(sub_index) > 0:
                    index.add('{}{}'.format(path, entry.name).replace(posts_path, ''))
                    index = index.union(sub_index)

            if entry.is_file() and entry.name.endswith('.md'):
                index.add(path.replace(posts_path, '') + entry.name[:len(entry.name)-3])

    return index


post_index = sorted(generate_index(posts_path))
with open('post_index', 'w') as f:
    f.write(json.dumps(post_index))
    f.close()

# for entry in post_index:
#     fname = '{}{}.md'.format(posts_path, entry)
#     try:
#         with open(fname, 'r'):
#             print(fname)
#             f.close()
#     except FileNotFoundError:
#         continue
