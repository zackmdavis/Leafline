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

UNDERSCORE_PATH = os.path.join('web_client', 'resources', 'public', 'js',
                               "underscore-min.js")

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
    boardzip = zipfile.ZipFile(CHESSBOARDJS_ZIPBALL_DOWNLOAD_PATH)
    for name in boardzip.namelist():
        if name.startswith("js") or name.startswith("css"):
            boardzip.extract(name, path=os.path.join('web_client', 'resources', 'public'))

@task
def install_chessboard_js():
    download_chessboard_js()
    unpack_chessboard_js()


@not_if_files_exist(UNDERSCORE_PATH)
def download_underscore():
    urlretrieve("http://underscorejs.org/underscore-min.js", UNDERSCORE_PATH)


@task
def download_statics():
    install_chessboard_js()
    download_underscore()


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
@not_if_files_exist(*[os.path.join('src', f)
                      for f in ("motion.rs", "landmark.rs")])
def build_furniture():
    tablemaker.main()


@task
def build_release():
    run("cargo build --release")
    run(' '.join(segment for segment in BABEL_COMMAND if segment != "--watch"))
    run("cd web_client && lein uberjar")
    run("cp target/release/leafline provisioning/leafline")
    run("cp web_client/target/leafline-web-client.jar "
        "provisioning/leafline-web-client.jar")

@task
def sed(pattern, replacement):
    for subtree in ('src', "web_client"):
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


@task
def new_methodize(struct_name, src_file=None):
    field_subliteral_pattern = "(\w+): ([^\s,]+),?\s+"
    field_subliteral_regex = re.compile(field_subliteral_pattern)
    literal_pattern = r"{} ?{{ ({})+}}".format(
        struct_name, field_subliteral_pattern)
    literal_regex = re.compile(literal_pattern)

    if src_file is None:
        filepaths = [os.path.join('src', name) for name in os.listdir('src')]
    else:
        filepaths = [os.path.join('src', src_file)]

    for filepath in filepaths:
        if not filepath.endswith(".rs"):
            continue
        with open(filepath) as source_file:
            source = source_file.read()
        for struct_literal_match in literal_regex.finditer(source):
            new_args =  ', '.join(subliteral_match.group(2)
                                  for subliteral_match
                                  in field_subliteral_regex.finditer(
                                      struct_literal_match.group(0)))
            new_call = "{}::new({})".format(struct_name, new_args)
            source = source.replace(struct_literal_match.group(0), new_call)
        with open(filepath, 'w') as source_file:
            source_file.write(source)
