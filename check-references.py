import fileinput
import os
import re

for line in fileinput.input():
    with open(line.strip(), 'r') as file:
        for match in re.findall(r'\[.+?]\(((?!http).+?)\)', file.read(), re.MULTILINE):
            if match.startswith('#'):
                parts = match[1:].split(':')
                markdownFilePath = 'posts/{}.md'.format(parts[0])
                try:
                    with open(markdownFilePath, 'r') as markdownFile:
                        markdownContent = re.sub(r'^```(.*?)```$', '', markdownFile.read(), 0, re.MULTILINE | re.DOTALL)
                        headlines = []
                        for headline in re.findall(r'^[#]{1,5} (.+?)$', markdownContent, re.MULTILINE):
                            headlines.append(headline.replace(' ', '_'))

                        if parts[1] not in headlines:
                            print('invalid reference: {} in {}'.format(match, line.strip()))
                except FileNotFoundError:
                    print('invalid reference: {} in {}'.format(match, line.strip()))
            else:
                if match.startswith('/'):
                    match = '.{}'.format(match)

                if not os.path.isfile(match):
                    print('missing file: {} in {}'.format(match, line.strip()))
    pass
