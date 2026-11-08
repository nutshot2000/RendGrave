"""RENDGRAVE - title theme.

Concept: the first thing you hear is a lone bell tolling over a dark forest at night.
A slow 6/8 (a dead man's waltz) in D minor. The theme is grave but heroic: it rises,
reaches for a high note and turns away, like hope that won't quite commit.
Form (32 bars, ~68 s, loops):  toll (8) > theme (8) > theme with the march (8) > fade back to the toll (8)
No long low notes: bass is short plucks, the pad never sits under C4.
"""
import os
from rgmidi import Song, P

s = Song(bpm=84, steps_per_bar=6, steps_per_beat=2)   # 6/8: steps are 8th notes
LEAD, HARP, BASS, BELL, STR, DR = 0, 1, 2, 3, 4, 9
s.program(LEAD, 60)   # horn
s.program(HARP, 46)   # harp
s.program(BASS, 32)   # acoustic bass
s.program(BELL, 14)   # tubular bell
s.program(STR, 48)    # strings

CH = {  # bass root, harp tones low->high
    "Dm": ("D2", ["D4", "F4", "A4", "D5"]),
    "Bb": ("Bb1", ["D4", "F4", "Bb4", "D5"]),
    "Gm": ("G1", ["D4", "G4", "Bb4", "D5"]),
    "C": ("C2", ["E4", "G4", "C5", "E5"]),
    "A": ("A1", ["C#4", "E4", "A4", "C#5"]),
    "Asus": ("A1", ["D4", "E4", "A4", "D5"]),
    "Eb": ("Eb2", ["Eb4", "G4", "Bb4", "Eb5"]),
    "F": ("F2", ["C4", "F4", "A4", "C5"]),
}


def harp(bar, c, vel=58, shape="roll"):
    t = CH[c][1]
    order = {"roll": [0, 1, 2, 3, 2, 1], "fall": [3, 2, 1, 0, 1, 2], "rise": [0, 1, 2, 3, 3, 3]}[shape]
    for k, i in enumerate(order):
        s.note(HARP, bar, k, t[i], 1, vel - (8 if k % 3 else 0))


def bass(bar, c, vel=80):
    r = P(CH[c][0])
    s.note(BASS, bar, 0, r, 2, vel)
    s.note(BASS, bar, 3, r + 7, 1, vel - 20)


def toll(bar, vel=90):
    s.note(BELL, bar, 0, "D5", 2, vel)
    s.note(BELL, bar, 2, "D5", 1, vel // 3)   # the echo


def march(bar, last=False):
    s.note(DR, bar, 0, 41, 1, 92)
    s.note(DR, bar, 3, 43, 1, 64)
    if last:
        for k, v in ((4, 60), (5, 80)):
            s.note(DR, bar, k, 45, 1, v)


THEME = [  # (pitch, 8ths)
    [("A4", 3), ("D5", 2), ("E5", 1)],
    [("F5", 4), ("E5", 1), ("D5", 1)],
    [("D5", 3), ("F5", 3)],
    [("E5", 6)],
    [("G5", 3), ("F5", 2), ("E5", 1)],
    [("F5", 3), ("D5", 3)],
    [("E5", 2), ("D5", 2), ("C#5", 2)],
    [("E5", 6)],
]
THEME_CH = ["Dm", "Dm", "Bb", "C", "Gm", "Dm", "Bb", "A"]
# second statement reaches higher and lands on the flat-two (Eb) before turning back
THEME2 = [
    [("A4", 3), ("D5", 2), ("E5", 1)],
    [("F5", 4), ("G5", 1), ("A5", 1)],
    [("Bb5", 3), ("A5", 2), ("G5", 1)],
    [("A5", 4), ("G5", 1), ("F5", 1)],
    [("G5", 3), ("Bb5", 2), ("D6", 1)],
    [("C6", 3), ("Bb5", 2), ("G5", 1)],
    [("A5", 2), ("G5", 2), ("E5", 2)],
    [("C#5", 3), ("E5", 3)],
]
THEME2_CH = ["Dm", "Dm", "Gm", "F", "Gm", "Eb", "A", "A"]
COUNTER = [  # strings answer in the gaps, a third or sixth below
    [(0, 3), ("F4", 3)],
    [("A4", 4), (0, 2)],
    [("Bb4", 3), ("D5", 3)],
    [("C5", 3), ("Bb4", 3)],
    [("D5", 3), (0, 3)],
    [("Eb5", 3), ("D5", 3)],
    [("C#5", 2), ("Bb4", 2), ("G4", 2)],
    [("A4", 3), ("G4", 3)],
]

bar = 0
# --- toll: bell, harp falling in, the bass waking up
intro = ["Dm", "Dm", "Dm", "Dm", "Bb", "Gm", "Asus", "A"]
for i, c in enumerate(intro):
    if i % 2 == 0 or i >= 4:
        toll(bar + i, 92 - i * 4)
    if i >= 2:
        harp(bar + i, c, 46 + i * 2, "fall" if i < 6 else "rise")
    if i >= 4:
        bass(bar + i, c, 70)
bar += 8
# --- theme: horn alone over the harp
for i, c in enumerate(THEME_CH):
    s.seq(LEAD, bar + i, 0, THEME[i], 96)
    harp(bar + i, c, 56)
    bass(bar + i, c)
    if i == 0:
        toll(bar + i, 70)
bar += 8
# --- theme again, higher, strings answering, a funeral march underneath
for i, c in enumerate(THEME2_CH):
    s.seq(LEAD, bar + i, 0, THEME2[i], 104)
    s.seq(STR, bar + i, 0, COUNTER[i], 70)
    harp(bar + i, c, 60, "roll")
    bass(bar + i, c, 88)
    march(bar + i, last=(i == 7))
    if i in (0, 4):
        toll(bar + i, 84)
bar += 8
# --- fade: fragments, the bell returns, a rising harp hands back to the start
fade = ["Bb", "F", "Gm", "A", "Dm", "Bb", "Asus", "A"]
frag = [[("D5", 3), ("C5", 3)], [("C5", 3), ("A4", 3)], [("Bb4", 6)], [("A4", 6)], [], [("F4", 3), ("E4", 3)], [], []]
for i, c in enumerate(fade):
    s.seq(LEAD, bar + i, 0, frag[i], 78 - i * 4)
    harp(bar + i, c, 52 - i * 2, "rise" if i == 7 else "fall")
    if i < 6:
        bass(bar + i, c, 70 - i * 4)
    if i >= 4:
        toll(bar + i, 80)
bar += 8

s.write(os.path.join(os.path.dirname(os.path.abspath(__file__)), "rg_title.mid"), bar)
