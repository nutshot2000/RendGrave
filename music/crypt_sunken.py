"""THE SUNKEN CRYPT - dungeon.

Concept: you are somewhere you should not be, and something is asleep down here.
The music is mostly silence. A heartbeat kick (lub-dub) is your own pulse. A plucked bass
creeps down by half steps, one short note at a time, never sustaining. A bell drops a
tritone (D to G#), the sound of the place itself. Later a far-off organ breathes
diminished chords and a pan flute sighs in half steps. Built on the octatonic scale so
nothing ever resolves.
Form (20 bars of 4/4, ~73 s, loops): creep (8) > the deep (8) > breath (4)
No drones: every note is short, with silence after it.
"""
import os
from rgmidi import Song, P

s = Song(bpm=66, steps_per_bar=16, steps_per_beat=4)
FLUTE, BELL, ORGAN, BASS, DR = 0, 1, 2, 3, 9
s.program(FLUTE, 75)   # pan flute
s.program(BELL, 9)     # glockenspiel
s.program(ORGAN, 19)   # church organ
s.program(BASS, 45)    # pizzicato


def heart(bar, vel=96):
    s.note(DR, bar, 0, 36, 1, vel)
    s.note(DR, bar, 3, 36, 1, vel - 30)


def tick(bar, vel=34):
    for st in (4, 8, 12):
        s.note(DR, bar, st, 42, 1, vel)


def creep(bar, root, nxt, vel=78):
    r, n = P(root), P(nxt)
    s.note(BASS, bar, 0, r, 2, vel)
    s.note(BASS, bar, 8, r + 12, 1, vel - 30)
    s.note(BASS, bar, 14, n + (1 if n < r else -1), 1, vel - 20)   # leans into the next note


def toll(bar, hi="D6", lo="G#5", vel=70):
    s.note(BELL, bar, 0, hi, 2, vel)
    s.note(BELL, bar, 2, lo, 3, vel - 10)
    s.note(BELL, bar, 8, hi, 1, vel // 3)       # echo off the stone
    s.note(BELL, bar, 9, lo, 1, vel // 4)


CREEP = ["D3", "C#3", "C3", "B2", "Bb2", "A2", "G#2", "A2", "D3"]
DIM = [["D5", "F5", "G#5"], ["C#5", "E5", "G5"], ["C5", "Eb5", "F#5"], ["B4", "D5", "F5"]]
SIGH = [
    [(0, 4), ("A5", 6), ("Bb5", 2), ("A5", 4)],
    [("F5", 8), (0, 8)],
    [(0, 4), ("G#5", 6), ("A5", 2), ("G#5", 4)],
    [("E5", 8), (0, 8)],
    [(0, 4), ("D6", 4), ("C#6", 4), ("C6", 4)],
    [("B5", 4), ("Bb5", 4), ("A5", 8)],
    [(0, 8), ("F5", 4), ("E5", 4)],
    [("D5", 8), (0, 8)],
]

bar = 0
for i in range(8):                       # the creep: pulse, bass, the bell
    heart(bar + i, 80 + i * 2)
    if i >= 2:
        creep(bar + i, CREEP[i - 2], CREEP[i - 1])
    if i % 2 == 0:
        toll(bar + i, vel=64 + i * 2)
    if i >= 4:
        tick(bar + i)
bar += 8
deep_bass = ["D3", "G#2", "C#3", "G2", "C3", "F#2", "B2", "F2", "D3"]
for i in range(8):                       # the deep: organ far away, the flute sighing
    heart(bar + i, 98)
    tick(bar + i, 40)
    creep(bar + i, deep_bass[i], deep_bass[i + 1], 84)
    s.chord(ORGAN, bar + i, 0, DIM[i % 4], 6, 46)
    s.chord(ORGAN, bar + i, 8, DIM[(i + 1) % 4], 4, 38)
    s.seq(FLUTE, bar + i, 0, SIGH[i], 82)
    if i in (3, 7):
        toll(bar + i, "C#6", "G5", 60)
    if i == 7:
        for st, p in ((12, 45), (14, 43)):
            s.note(DR, bar + i, st, p, 1, 70)
bar += 8
for i in range(4):                       # breath: almost nothing, then the pulse returns
    if i == 0:
        toll(bar + i, vel=72)
    if i == 2:
        toll(bar + i, "G#5", "D5", 54)
    if i >= 2:
        heart(bar + i, 60 + i * 8)
    if i == 3:
        s.note(BASS, bar + i, 14, "D#3", 1, 60)
bar += 4

s.write(os.path.join(os.path.dirname(os.path.abspath(__file__)), "rg_crypt.mid"), bar)
