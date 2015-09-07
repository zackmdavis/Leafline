import os
import subprocess
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


@task
def compile_client():
    subprocess.check_output(
        ["babel",
         "web_client/resources/public/js/client.js",
         "--watch",
         "--out-file",
         "web_client/resources/public/js/client-built.js"]
    )
