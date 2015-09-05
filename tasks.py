import os
from urllib.request import urlretrieve

from invoke import task, run


@task
def install_chessboard_js():
    urlretrieve("http://chessboardjs.com/releases/0.3.0/chessboardjs-0.3.0.zip",
                # TODO: current-working-directory robustness
                os.path.join('web_client', 'resources',
                             'chessboardjs-0.3.0.zip'))
    # TODO: unzip JS and CSS to appropriate places


@task
def download_statics():
    install_chessboard_js()
