#!/usr/bin/env python3

import argparse
import ctypes
from ctypes import c_byte, c_char_p, c_float
import os


class EmbassyException(Exception):
    pass


class Scoring(ctypes.Structure):
    _fields_ = [
        ('movement', c_byte * 10),
        ('score', c_float)
    ]

    def render(self):
        return (bytes(self.movement).decode().strip('\x00'), self.score)


def render_scoring_array(scoring_array):
    return [scoring.render() for scoring in scoring_array
            if scoring.render() != ('', 0.0)]

ScoringArray = Scoring * 60
ScoringArray.render = render_scoring_array


LIBRARY_LOCATIONS = [
    "./libleafline.so",
    "./target/release/libleafline.so",
    "./target/debug/libleafline.so",
]

for location in LIBRARY_LOCATIONS:
    if os.path.exists(location):
        libleafline = ctypes.CDLL(location)
        break
else:
    raise EmbassyException("couldn't locate libleafline.so!")
libleafline.score.argtypes = (c_char_p, c_byte, ScoringArray)


def score(preservation_runes, depth):
    scorings = ScoringArray()
    result = libleafline.score(preservation_runes.encode(), depth, scorings)
    return scorings.render()


if __name__ == "__main__":
    arg_parser = argparse.ArgumentParser()
    arg_parser.add_argument(
        "--from", type=str, required=True,
        help="score from the given book of preservation runes")
    arg_parser.add_argument(
        "--depth", type=int, required=True,
        help="rank moves using AI minimax lookahead this deep")
    args = arg_parser.parse_args()
    print(score(getattr(args, 'from'), args.depth))
