#!/usr/bin/env python3

import itertools
from functools import reduce
import operator
import os

def rank_and_file_to_u64(position):
    rank, file = position
    pindex = (8 * rank) + file
    return 2**pindex

def displace(position, offset):
    return tuple(map(operator.add, position, offset))

def is_legal(position):
    return all(0 <= directional < 8 for directional in position)


def center_of_the_world():
    return reduce(operator.ior,
                  [rank_and_file_to_u64(position)
                   for position in itertools.product(range(2, 6), repeat=2)])

PONY_OPTIONS = ((+1, +2), (-1, +2), (+1, -2), (-1, -2),
                (+2, +1), (-2, +1), (+2, -1), (-2, -1))

FIGUREHEAD_OPTIONS = [(i, j)
                      for i in (-1, 0, 1)
                      for j in (-1, 0, 1)
                      if not i == j == 0]


def universal_distribution(options):
    return [reduce(operator.ior,
                   [rank_and_file_to_u64(displace(position, offset))
                    for offset in options
                    if is_legal(displace(position, offset))])
            for position in itertools.product(range(8), repeat=2)]

def the_book_of_life(job_description, result):
    return "pub static {}_MOVEMENT_TABLE: [u64; 64] = [\n{}\n];\n".format(
        job_description.upper(),
        '\n'.join("    {},".format(entry) for entry in result)
    )


if __name__ == "__main__":
    with open(os.path.join('src', 'motion.rs'),
              'w') as motion_rs:
        motion_rs.write(
            '\n\n'.join(
                ["pub static CENTER_OF_THE_WORLD: u64 = {};".format(
                    center_of_the_world()),
                 the_book_of_life(
                     "pony", universal_distribution(PONY_OPTIONS)),
                 the_book_of_life(
                     "figurehead", universal_distribution(FIGUREHEAD_OPTIONS))]
            )
        )
    print("Wrote motion.rs!")
