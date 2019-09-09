import subprocess
import os

import json

"""
git-annex utilities. All functions in this module expect the pwd to be located in the git repo we're operating on.
"""

def annex_add(filelist):
    """ git annex add a list of files
    Returns a list of dicts:
    out['success'] :: Bool -- Was the call successful
    out['key'] :: String -- Key of the annexed document
    out['error-messages'] :: [String] -- Error message if success=False
    out['file'] :: Path -- Filename that was added
    """
    stdin = ""
    for file in filelist:
        stdin += file + "\n"

    stdout = subprocess.check_output("git annex add --json --json-error-messages --batch", shell=True, input=stdin.encode('utf-8'))

    lines = stdout.splitlines()
    out = []
    for line in lines:
        if line != b'':
            j = json.loads(line)
            out.append(j)
        else:
            out.append({})

    return out


def annex_sync(content):
    subprocess.check_output("git annex sync %s" % ("--content" if content else ""), shell=True)
