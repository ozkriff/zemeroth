#!/usr/bin/env python3
"""Calculates `md5` hash of the assets and writes it to `.checksum.md5` file.

Filenames are hashed too.
Hidden files (including `.checksum.md5` and `.travis.yml`) are ignored.
Python 3.4 required.
"""

import sys
import os
import hashlib
import argparse

assert sys.version_info >= (3, 4), str(sys.version_info)

def file_list():
    matches = []
    for root, dirnames, filenames in os.walk('.', topdown=True):
        # Ignore hidden files and dirs
        filenames = [f for f in filenames if not f[0] == '.']
        dirnames[:] = [d for d in dirnames if not d[0] == '.']
        for filename in filenames:
            matches.append(os.path.join(root, filename))
    return matches

parser = argparse.ArgumentParser()
parser.add_argument("-c", "--check", action='store_true')
args = parser.parse_args()

constructor = hashlib.md5()
for file_name in sorted(file_list()):
    if os.path.isfile(file_name):
        print("Hashing '{}'...".format(file_name));
        constructor.update(file_name.encode())
        constructor.update(open(file_name, 'rb').read())

hash = constructor.hexdigest()
print("The hash is {}".format(hash))

if args.check:
    print("Comparing the hash with `.checksum.md5`...")
    saved_hash = open('.checksum.md5').read().rstrip()
    if hash != saved_hash:
        exit("[ERROR] Hashes don't match!")
    print("Hashes match")
else:
    print("Updating `.checksum.md5`...")
    with open('.checksum.md5', mode='w') as f:
        f.write(hash + '\n')
