# coding: utf-8

import os
import subprocess
import sys
import time

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

# TODO: current-working-directory robustness

@task
def install_chessboard_js():
    urlretrieve("http://chessboardjs.com/releases/0.3.0/chessboardjs-0.3.0.zip",

                os.path.join('web_client', 'resources',
                             'chessboardjs-0.3.0.zip'))
    # TODO: unzip JS and CSS to appropriate places


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
