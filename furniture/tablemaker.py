#!/usr/bin/env python3

import itertools
from functools import reduce
import operator
import os

def rank_and_file_to_u64(position):
    rank, file = position
    bit_index = (8 * rank) + file
    return 2**bit_index

def displace(position, offset):
    return tuple(map(operator.add, position, offset))

def is_legal(position):
    return all(0 <= directional < 8 for directional in position)

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
    with open(os.path.join('src', 'movement_tables.rs'),
              'w') as movement_tables_rs:
        movement_tables_rs.write(
            '\n\n'.join(
                [the_book_of_life(
                    "pony", universal_distribution(PONY_OPTIONS)),
                 the_book_of_life(
                     "figurehead", universal_distribution(FIGUREHEAD_OPTIONS))]
            )
        )
    print("Wrote movement_tables.rs!")
