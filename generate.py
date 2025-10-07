import collections
import os, json

from git import Repo, NULL_TREE

posts_path = 'posts'
update_diffs_path = 'update_diffs'

if posts_path[len(posts_path) - 1] != '/':
    posts_path += '/'


def generate_index(path):
    index = set()
    with os.scandir(path) as it:
        for entry in it:
            if entry.is_dir():
                sub_index = generate_index(f'{path}{entry.name}/')
                if len(sub_index) > 0:
                    index.add(f'{path}{entry.name}'.replace(posts_path, ''))
                    index = index.union(sub_index)

            if entry.is_file() and entry.name.endswith('.md'):
                index.add(path.replace(posts_path, '') + entry.name[:len(entry.name)-3])

    return index


post_index = sorted(generate_index(posts_path))
with open('post_index.json', 'w') as f:
    f.write(json.dumps(post_index))
    f.close()

for filename in os.listdir(update_diffs_path):
    os.unlink(os.path.join(update_diffs_path, filename))

changelist = collections.OrderedDict()
repo = Repo(posts_path)
previous_commit = None
for commit in repo.iter_commits():
    commit_changelist = []
    commit_diffs = []
    for diff in commit.diff(other=(NULL_TREE if previous_commit is None else previous_commit), create_patch=True):

        affected_file = diff.b_path + ('' if diff.a_path is None or diff.a_path == diff.b_path else ':' + diff.a_path)
        if not affected_file.endswith('.md'):
            continue
        
        change = 'content'
        if diff.new_file:
            change = 'new'
        elif diff.renamed_file:
            change = 'renamed'
        elif diff.copied_file:
            change = 'copied'
        elif diff.deleted_file:
            change = 'deleted'

        commit_changelist.append({
            'change': change,
            'path': affected_file,
        })

        commit_diffs.append({
            'path': affected_file,
            'diff': diff.diff.decode('utf-8'),
        })

    if len(commit_changelist) > 0:
        timestamp = int(commit.committed_datetime.timestamp())
        changelist[timestamp] = commit_changelist

        with open(f'update_diffs/{timestamp}.json', 'w') as f:
            f.write(json.dumps(commit_diffs))
            f.close()

with open('updates.json', 'w') as f:
    f.write(json.dumps(changelist))
    f.close()
