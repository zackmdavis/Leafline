# coding: utf-8

import os
import re
import subprocess
import sys
import time
import zipfile
from functools import wraps

if sys.version_info < (3,):
    print("Are you using Python 2?")
    time.sleep(1.4)
    print("Do you know what year it is?")
    time.sleep(1.4)
    print("—okay.")
    time.sleep(1.4)
    from urllib import urlretrieve
else:
    from urllib.request import urlretrieve

from invoke import task, run

from furniture import tablemaker


# TODO: current-working-directory robustness


def not_if_files_exist(*filenames):
    def derived_decorator(func):
        @wraps(func)
        def core(*args, **kwargs):
            inventory = {f: os.path.exists(f) for f in filenames}
            if all(inventory.values()):
                print("skipping {}: {} already present".format(
                    func.__name__, ', '.join(filenames)))
            else:
                print("running {}".format(func.__name__))
                return func(*args, **kwargs)
        return core
    return derived_decorator


CHESSBOARDJS_ZIPBALL_DOWNLOAD_PATH = os.path.join('web_client', 'resources',
                                                  'chessboardjs-0.3.0.zip')


@not_if_files_exist(CHESSBOARDJS_ZIPBALL_DOWNLOAD_PATH)
def download_chessboard_js():
    urlretrieve("http://chessboardjs.com/releases/0.3.0/chessboardjs-0.3.0.zip",
                CHESSBOARDJS_ZIPBALL_DOWNLOAD_PATH)

@not_if_files_exist(*[
   os.path.join('web_client', 'resources', 'public', subpath)
    for subpath in
    (os.path.join('css', "chessboard-0.3.0.min.css"),
     os.path.join('js', "chessboard-0.3.0.min.js"))
])
def unpack_chessboard_js():
    "#TODO"


@task
def install_chessboard_js():
    download_chessboard_js()
    unpack_chessboard_js()


@task
def download_statics():
    install_chessboard_js()


BABEL_COMMAND = [
    "babel", "web_client/resources/public/js/client.js",
    "--watch",
    "--out-file", "web_client/resources/public/js/client-built.js"
]

@task
def compile_client():
    subprocess_runner = (subprocess.run if sys.version_info >= (3, 5)
                         else subprocess.call)
    subprocess_runner(BABEL_COMMAND)


@task
@not_if_files_exist(os.path.join('src', 'motion.rs'))
def build_furniture():
    tablemaker.main()


@task
def sed(pattern, replacement):
    for subtree in ("src", "web_client"):
        for fortress, _subsubtrees, deëdgers in os.walk(subtree):
            for deëdger in deëdgers:
                with open(os.path.join(fortress, deëdger), 'r+') as d:
                    try:
                        prior = d.read()
                    except UnicodeDecodeError:
                        ...
                    else:
                        posterior = re.sub(pattern, replacement, prior)
                        if prior != posterior:
                            d.seek(0)
                            d.write(posterior)
                            d.truncate()
