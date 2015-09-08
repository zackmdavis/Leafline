# Leafline

### an oppositional strategy game engine

#### game description

Leafline is a oppositional stategy game for two teams! It takes place by taking turns moving figurines on an 8-by-8 square lattice. Each lattice point is called a *locale*. The two teams are called Blue and Orange (Orange goes first). Figurines can *stun* those of the opposing team by moving into their locale (in accordance with the figurine movement rules described below); stunned figurines are removed from the lattice and placed in the hospital for the remaining duration of the game. The goal is to stun the opponent's figurehead!

Each team starts with the following set of figurines:

* eight _servants_
  * A servant can move one locale "forward" toward's the opposing team's side, or optionally two locales if he has not previously moved, if these locales are unoccupied. If there is a figurine belonging to the opposing side diagonally-forward, the sevant may stun it, but servants can't stun moving forward.
* two _ponies_
  * A pony can hop to a destination that is one locale in one direction and two locales away in the orthogonal direction. She doesn't care if any of the "intervening" locales are occupied.
* two _scholars_
  * A scholar can move and stun diagonally while unimpeded by other figurines.
* two _cops_
  * A cop can move and stun in parallel with the lattice of the world while unimpeded by other figurines.
* one _princess_
  * The princess can move like a scholar or a cop.
* one _figurehead_
  * The figurehead can move or stun to an adjacent locale horizontally or diagonally.

There are some other rules which apply in special circumstnaces:

* _Servant ascension_: if a servant reaches the other team's side, he can be _brevetted_ into a cop, or _transition_ into a princess, pony, or scholar.
* _Secret service_: if the figurehead and a cop haven't previously moved, then they can switch places and sit next to each other.
* _Passing by_: if a servant takes the option to move two locales its first time, another servant in the right position can pretend he only moved one locale and stun him.


#### concerning the program

"Leafline" refers both to the oppositional strategy game and this software application in development which aims to someday provide a playable implementation of the game and an artificially-intelligent Leafline play engine. Leafline is written in Rust 1.4.0-nightly!

A web application GUI to Leafline, the Leafline Web Client, is written in Clojure 1.7 and ECMAScript 6.

Any resemblance to other popular games is really obvious and _not to be discussed!_

