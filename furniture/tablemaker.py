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

PONY_RESULT = [reduce(operator.ior,
                     [rank_and_file_to_u64(displace(position, offset))
                      for offset in PONY_OPTIONS
                      if is_legal(displace(position, offset))])
              for position in itertools.product(range(8), repeat=2)]

def embeddable_result(result):
    return "pub static PONY_MOVEMENT_TABLE: [u64; 64] = [\n{}\n];\n".format(
        '\n'.join("    {},".format(entry) for entry in result)
    )


if __name__ == "__main__":
    with open(os.path.join('src', 'movement_tables.rs'),
              'w') as movement_tables_rs:
        movement_tables_rs.write(embeddable_result(PONY_RESULT))
    print("Wrote movement_tables.rs!")
