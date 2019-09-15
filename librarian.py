#!/usr/bin/env python

import argparse
import configparser
import os

import pathlib

import importlib

import magic

import git

from cloudant import couchdb as CouchDB
from cloudant.database import CouchDatabase as Database
from cloudant.document import Document

from mime.registry import Registry

from yaml import load, dump

cfgpath = os.path.expanduser("~/.config/librarian.cfg")

parser = argparse.ArgumentParser(prog="librarian", description="Archive the metadata of all files listed")
parser.add_argument('--config')
parser.set_defaults(config=cfgpath)

subparsers = parser.add_subparsers()

registry = Registry()

def archive(args):
    """archive a list of files"""

    res = git.annex_add(args.files)
    with CouchDB(config["auth"]["user"], config["auth"]["pass"], url="http://127.0.0.1:5984", connect=True) as client:
        db = Database(client, args.db)

        for r in res:
            if r != {} and r['success']:

                ext = pathlib.PurePosixPath(r['file']).suffix

                mimetype = mime_from_ext(ext)
                if not mimetype:
                    f = open(r['file'], 'rb')
                    mimetype = magic.from_buffer(f.read(1024), mime=True)

                mimemod = mimetype.replace("/", ".")

                try:
                    pkg = importlib.import_module("mime." + mimemod)

                    doc = Document(db, document_id=r['key'])
                    if doc.exists():
                        doc.fetch()

                    pkg.archive(doc, r['file'])

                    doc.save()

                except Exception as e:
                    print("Exception %s for handler %s when archiving %s" % (e, mimemod, r['file']))

archivep = subparsers.add_parser('archive', help=archive.__doc__)
archivep.add_argument('db', metavar='database', help='the CouchDB database the files should be archived into.')
archivep.add_argument('files', metavar='filename', nargs='+', help='a file to be archived.')
archivep.set_defaults(func=archive)

def mime_from_ext(ext):
    if ext == ".mp3":
        return "audio/mpeg"

def query(args):
    """query metadata"""
    print("Called the 'query' subcommand with args %s" % args)

queryp = subparsers.add_parser('query', help=query.__doc__)
queryp.set_defaults(func=query)

def sync(args):
    """sync metadata and files"""
    git.annex_sync(args.content)


syncp = subparsers.add_parser('sync', help=sync.__doc__)
syncp.add_argument('--content', help='a file to be archived.', action='store_true')
syncp.set_defaults(func=sync)


def read(args):
    """read metadata from a list of files"""
    for f in args.files:
        ext = pathlib.PurePosixPath(f).suffix

        mimetype = mime_from_ext(ext)
        if not mimetype:
            fp = open(f, 'rb')
            mimetype = magic.from_buffer(fp.read(1024), mime=True)

        mimemod = mimetype.replace("/", ".")

        try:
            pkg = importlib.import_module("mime." + mimemod)

            doc = {}

            pkg.archive(doc, f)

            print(doc)

        except Exception as e:
            print("Exception %s for handler %s when archiving %s" % (e, mimemod, f))


readp = subparsers.add_parser('read', help=read.__doc__)
readp.add_argument('files', metavar='filename', nargs='+', help='a file to be archived.')
readp.set_defaults(func=read)

args = parser.parse_args()

config = configparser.ConfigParser()
config.read(str(args.config))

args.func(args)
