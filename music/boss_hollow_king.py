"""THE HOLLOW KING - act I boss.

Concept: a dead king who still believes he rules. It opens like a coronation, brass and organ,
then breaks into a gallop: everything in triplets (12/8 feel), the sound of a cavalry charge
that died centuries ago. F minor with the Neapolitan G-flat (regal, wrong) and a C-major
dominant that drags every phrase back. The middle is his resurrection: half-time, a choir,
a chromatic E-major chord that should not be there. Then the gallop again.
Form (36 bars, ~63 s, loops): crown (4) > gallop (8) > frenzy (8) > resurrection (8) > gallop (8)
"""
import os
from rgmidi import Song, P

s = Song(bpm=138, steps_per_bar=12, steps_per_beat=3)   # triplet 8ths: 12/8 feel
BRASS, ORGAN, BASS, CHOIR, SAW, DR = 0, 1, 2, 3, 4, 9
s.program(BRASS, 61)
s.program(ORGAN, 19)
s.program(BASS, 34)
s.program(CHOIR, 52)
s.program(SAW, 81)

CH = {  # bass root, organ voicing
    "Fm": ("F2", ["F4", "Ab4", "C5"]),
    "Db": ("Db2", ["F4", "Ab4", "Db5"]),
    "Eb": ("Eb2", ["G4", "Bb4", "Eb5"]),
    "Gb": ("Gb2", ["Gb4", "Bb4", "Db5"]),
    "C": ("C2", ["E4", "G4", "C5"]),
    "Bbm": ("Bb1", ["F4", "Bb4", "Db5"]),
    "E": ("E2", ["E4", "G#4", "B4"]),
}


def gallop(bar, c, vel=92):
    r = P(CH[c][0])
    for b in range(4):
        hi = r + (12 if b % 2 else 0)
        s.note(BASS, bar, b * 3, hi, 2, vel if b % 2 == 0 else vel - 12)
        s.note(BASS, bar, b * 3 + 2, hi, 1, vel - 28)


def stabs(bar, c, vel=74, pat=((0, 2), (6, 2), (10, 1))):
    for st, n in pat:
        s.chord(ORGAN, bar, st, CH[c][1], n, vel)


def kit(bar, kind="gallop", fill=False, crash=False):
    if kind == "gallop":
        for st in (0, 2, 3, 5, 6, 8, 9, 11):
            s.note(DR, bar, st, 42, 1, 50 if st % 3 else 62)
        for st in (0, 6, 11):
            s.note(DR, bar, st, 36, 1, 112 if st != 11 else 80)
        for st in (3, 9):
            s.note(DR, bar, st, 38, 1, 106)
    elif kind == "half":
        s.note(DR, bar, 0, 36, 1, 108)
        s.note(DR, bar, 6, 38, 1, 100)
        for st in (0, 3, 6, 9):
            s.note(DR, bar, st, 51, 1, 46)                 # ride
    if fill:
        for k, p in enumerate((47, 47, 45, 45, 43, 43)):
            s.note(DR, bar, 6 + k, p, 1, 92 + k * 4)
    if crash:
        s.note(DR, bar, 0, 49, 3, 118)


CROWN = [   # the fanfare: fifths and a rising sixth, the crown motif
    [("F5", 3), ("C5", 2), ("F5", 1), ("Ab5", 6)],
    [("Db5", 3), ("Ab4", 2), ("Db5", 1), ("F5", 6)],
    [("Gb5", 3), ("Db5", 2), ("Gb5", 1), ("Bb5", 6)],
    [("G5", 2), ("Ab5", 1), ("Bb5", 2), ("C6", 1), (0, 6)],
]
A = [
    [("F5", 3), ("C5", 2), ("F5", 1), ("Ab5", 6)],
    [("G5", 2), ("F5", 1), ("Eb5", 3), ("C5", 6)],
    [("Db5", 3), ("F5", 2), ("Ab5", 1), ("Db6", 6)],
    [("C6", 2), ("Bb5", 1), ("Ab5", 3), ("G5", 6)],
    [("F5", 3), ("Ab5", 2), ("C6", 1), ("C6", 6)],
    [("Db6", 3), ("C6", 2), ("Bb5", 1), ("Gb5", 6)],
    [("E5", 2), ("F5", 1), ("G5", 3), ("Ab5", 2), ("G5", 1), ("F5", 3)],
    [("E5", 6), ("C5", 3), ("E5", 3)],
]
A_CH = ["Fm", "Fm", "Db", "Eb", "Fm", "Gb", "C", "C"]
B_CH = ["Bbm", "C", "Fm", "Db", "Bbm", "C", "Db", "C"]
RUN = {  # the frenzy: falling triplet arpeggios
    "Bbm": ["Bb5", "F5", "Db5"], "C": ["C6", "G5", "E5"], "Fm": ["C6", "Ab5", "F5"], "Db": ["Db6", "Ab5", "F5"],
}
B_BRASS = [  # brass punches between the runs
    [(0, 9), ("Bb5", 3)], [(0, 9), ("C6", 3)], [(0, 9), ("Ab5", 3)], [(0, 9), ("Ab5", 3)],
    [(0, 6), ("Db6", 3), ("Bb5", 3)], [(0, 6), ("E5", 3), ("G5", 3)], [("F5", 3), ("Ab5", 3), ("Db6", 6)], [("C6", 6), (0, 6)],
]
C_CH = ["Fm", "E", "Fm", "Gb", "Fm", "E", "Db", "C"]
CHOIR_PART = [  # resurrection: high, short, one note per half bar with breath between
    [("Ab5", 5), (0, 1), ("C6", 5), (0, 1)],
    [("G#5", 5), (0, 1), ("B5", 5), (0, 1)],
    [("Ab5", 5), (0, 1), ("F5", 5), (0, 1)],
    [("Bb5", 5), (0, 1), ("Db6", 5), (0, 1)],
    [("C6", 5), (0, 1), ("Ab5", 5), (0, 1)],
    [("B5", 5), (0, 1), ("G#5", 5), (0, 1)],
    [("Ab5", 5), (0, 1), ("F5", 5), (0, 1)],
    [("G5", 5), (0, 1), ("E5", 5), (0, 1)],
]

bar = 0
for i, c in enumerate(["Fm", "Db", "Gb", "C"]):     # crown
    s.seq(BRASS, bar + i, 0, CROWN[i], 104)
    stabs(bar + i, c, 70, ((0, 5),))
    s.note(BASS, bar + i, 0, P(CH[c][0]) + 12, 2, 90)
    if i == 3:
        for st in range(6, 12):
            s.note(DR, bar + i, st, 38, 1, 60 + (st - 6) * 10)
bar += 4
for i, c in enumerate(A_CH):                          # gallop
    s.seq(BRASS, bar + i, 0, A[i], 106)
    stabs(bar + i, c)
    gallop(bar + i, c)
    kit(bar + i, fill=(i == 7), crash=(i in (0, 4)))
bar += 8
for i, c in enumerate(B_CH):                          # frenzy
    run = RUN[c]
    for k in range(12):
        oct_drop = 12 if k >= 6 and i % 2 else 0
        s.note(SAW, bar + i, k, P(run[k % 3]) - oct_drop, 1, 82 if k % 3 == 0 else 66)
    s.seq(BRASS, bar + i, 0, B_BRASS[i], 108)
    stabs(bar + i, c, 70, ((2, 1), (5, 1), (8, 1), (11, 1)))
    gallop(bar + i, c, 96)
    kit(bar + i, fill=(i == 7), crash=(i == 0))
bar += 8
for i, c in enumerate(C_CH):                          # resurrection
    s.seq(CHOIR, bar + i, 0, CHOIR_PART[i], 74)
    stabs(bar + i, c, 60, ((0, 3),))
    s.note(BASS, bar + i, 0, P(CH[c][0]) + 12, 2, 86)
    s.note(BASS, bar + i, 6, P(CH[c][0]) + 12, 1, 64)
    kit(bar + i, "half", fill=(i == 7), crash=(i == 0))
    if i % 2 == 0:
        s.note(BRASS, bar + i, 0, "F4", 2, 88)        # a low brass knell under the choir
bar += 8
for i, c in enumerate(A_CH):                          # gallop again, saw doubling the brass
    s.seq(BRASS, bar + i, 0, A[i], 110)
    s.seq(SAW, bar + i, 0, A[i], 64, tr=-12)
    stabs(bar + i, c, 78)
    gallop(bar + i, c, 98)
    kit(bar + i, fill=(i == 7), crash=(i in (0, 4)))
bar += 8

s.write(os.path.join(os.path.dirname(os.path.abspath(__file__)), "rg_boss.mid"), bar)
