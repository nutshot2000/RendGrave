"""THE HOLLOW PINES - forest exploration.

Concept: adventurous but never comfortable. 5/4 felt as 3+2, so the beat always arrives a
little early, like a step you didn't expect. E minor with a Phrygian F natural for the
moments something moves in the dark. A plucked ostinato is the forest itself; an ocarina
is you. The B section lifts into C and D, the brave part, then a B-major chord yanks it
back to the dark. Drums are rims and hats, the forest floor, never a full kit.
Form (28 bars of 5/4, ~72 s, loops): ostinato (4) > A (8) > B (8) > A' (8)
"""
import os
from rgmidi import Song, P

s = Song(bpm=116, steps_per_bar=10, steps_per_beat=2)   # 5/4 on an 8th grid
LEAD, PLUCK, CLAR, BASS, DR = 0, 1, 2, 3, 9
s.program(LEAD, 79)    # ocarina
s.program(PLUCK, 45)   # pizzicato strings
s.program(CLAR, 71)    # clarinet
s.program(BASS, 32)

CH = {  # bass, (root, fifth, octave, colour)
    "Em": ("E2", ["E4", "B4", "E5", "G5"]),
    "C": ("C2", ["C4", "G4", "C5", "E5"]),
    "D": ("D2", ["D4", "A4", "D5", "F#5"]),
    "F": ("F2", ["F4", "C5", "F5", "A5"]),
    "G": ("G2", ["G4", "D5", "G5", "B5"]),
    "B": ("B1", ["B3", "F#4", "B4", "D#5"]),
    "Am": ("A2", ["A4", "E5", "A5", "C6"]),
}
PAT = [0, 1, 2, 1, 3, 0, 1, 2, 3, 1]   # 3+2 shape


def ostinato(bar, c, vel=60):
    t = CH[c][1]
    for k, i in enumerate(PAT):
        s.note(PLUCK, bar, k, t[i], 1, vel + (14 if k in (0, 6) else 0))


def bass(bar, c, vel=82):
    b = P(CH[c][0])
    s.note(BASS, bar, 0, b, 2, vel)
    s.note(BASS, bar, 6, b + 7, 2, vel - 14)


def floor(bar, kind, fill=False):
    for k in range(10):
        s.note(DR, bar, k, 42, 1, 40 if k % 2 else 52)        # hats
    s.note(DR, bar, 0, 37, 1, 84)                              # rim on the 3
    s.note(DR, bar, 6, 37, 1, 70)                              # rim on the 2
    if kind != "light":
        s.note(DR, bar, 0, 36, 1, 100)
    if kind == "brave":
        s.note(DR, bar, 6, 38, 1, 88)
        s.note(DR, bar, 3, 36, 1, 70)
    if fill:
        for k, p in ((7, 47), (8, 45), (9, 43)):
            s.note(DR, bar, k, p, 1, 96)


A = [  # (pitch, 8ths)
    [("B4", 3), ("E5", 1), ("G5", 2), ("F#5", 2), ("E5", 2)],
    [("B4", 6), (0, 4)],
    [("G5", 3), ("A5", 1), ("G5", 2), ("E5", 2), ("C5", 2)],
    [("D5", 6), (0, 2), ("F#5", 2)],
    [("G5", 3), ("F#5", 1), ("E5", 2), ("B5", 4)],
    [("A5", 2), ("G5", 2), ("F#5", 2), ("E5", 4)],
    [("F5", 3), ("E5", 1), ("D5", 2), ("C5", 2), ("A4", 2)],
    [("B4", 6), (0, 4)],
]
A_CH = ["Em", "Em", "C", "D", "Em", "Em", "F", "Em"]
B = [
    [("E5", 6), ("G5", 4)],
    [("F#5", 6), ("A5", 4)],
    [("B5", 6), ("A5", 2), ("G5", 2)],
    [("D5", 6), (0, 4)],
    [("E5", 3), ("G5", 3), ("C6", 4)],
    [("B5", 3), ("A5", 3), ("F#5", 4)],
    [("D#5", 6), ("F#5", 4)],
    [("B4", 4), ("D#5", 2), ("F#5", 2), ("A5", 2)],
]
B_CH = ["C", "D", "Em", "G", "C", "D", "B", "B"]
B_CLAR = [  # clarinet answers underneath in the gaps
    [(0, 6), ("C5", 2), ("E5", 2)],
    [(0, 6), ("D5", 2), ("F#5", 2)],
    [(0, 6), ("E5", 2), ("D5", 2)],
    [("B4", 4), ("G4", 2), ("B4", 2), ("D5", 2)],
    [(0, 6), ("E5", 2), ("G5", 2)],
    [(0, 6), ("F#5", 2), ("D5", 2)],
    [("B4", 6), ("D#5", 4)],
    [(0, 10)],
]
A2 = A[:4] + [
    [("C5", 3), ("E5", 1), ("A5", 2), ("G5", 2), ("E5", 2)],
    [("F5", 6), ("E5", 2), ("D5", 2)],
    [("D#5", 4), ("F#5", 2), ("B5", 4)],
    [("E5", 6), (0, 4)],
]
A2_CH = ["Em", "Em", "C", "D", "Am", "F", "B", "Em"]

bar = 0
for i, c in enumerate(["Em", "Em", "Em", "F"]):     # the forest before you arrive
    ostinato(bar + i, c, 50 + i * 4)
    if i >= 2:
        floor(bar + i, "light")
    if i == 3:
        bass(bar + i, c, 70)
bar += 4
for i, c in enumerate(A_CH):
    s.seq(LEAD, bar + i, 0, A[i], 94)
    ostinato(bar + i, c)
    bass(bar + i, c)
    floor(bar + i, "walk", fill=(i == 7))
bar += 8
for i, c in enumerate(B_CH):
    s.seq(LEAD, bar + i, 0, B[i], 100)
    s.seq(CLAR, bar + i, 0, B_CLAR[i], 70)
    ostinato(bar + i, c, 64)
    bass(bar + i, c, 88)
    floor(bar + i, "brave", fill=(i == 7))
bar += 8
for i, c in enumerate(A2_CH):
    s.seq(LEAD, bar + i, 0, A2[i], 92)
    ostinato(bar + i, c, 58)
    bass(bar + i, c, 78)
    floor(bar + i, "walk" if i < 7 else "light")
bar += 8

s.write(os.path.join(os.path.dirname(os.path.abspath(__file__)), "rg_pines.mid"), bar)
