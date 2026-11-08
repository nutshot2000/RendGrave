"""ASHFORD - the town theme.

Concept: the last safe hearth. A folk waltz around a campfire, the only warm music in the
game. G major with a Mixolydian F natural (an old, homely sound), 3/4 at an easy 104.
Plucked guitar does oom-pah-pah, a flute sings, a fiddle joins. The middle section slips
into E minor: someone glancing at the dark tree line past the palisade. Then it comes home.
Form (32 bars of 3/4, ~55 s, loops):  A flute > A with fiddle harmony > B minor-key > A full
"""
import os
from rgmidi import Song, P

s = Song(bpm=104, steps_per_bar=12, steps_per_beat=4)   # 3/4 on a 16th grid
FLUTE, FIDDLE, GTR, BASS, DR = 0, 1, 2, 3, 9
s.program(FLUTE, 73)
s.program(FIDDLE, 40)
s.program(GTR, 24)
s.program(BASS, 32)

CH = {  # bass, triad
    "G": ("G2", ["G3", "B3", "D4"]),
    "F": ("F2", ["F3", "A3", "C4"]),
    "C": ("C3", ["G3", "C4", "E4"]),
    "D": ("D2", ["F#3", "A3", "D4"]),
    "Em": ("E2", ["G3", "B3", "E4"]),
    "Am": ("A2", ["A3", "C4", "E4"]),
    "B": ("B2", ["B3", "D#4", "F#4"]),
}


def waltz(bar, c, vel=70):
    b, tri = CH[c]
    s.note(BASS, bar, 0, b, 3, vel + 10)
    for st in (4, 8):
        s.chord(GTR, bar, st, tri, 2, vel - 10)


def arp(bar, c, vel=58):
    b, tri = CH[c]
    s.note(BASS, bar, 0, b, 3, vel + 12)
    tones = [tri[0], tri[1], tri[2], P(tri[1]) + 12, tri[2], tri[1]]
    for k, p in enumerate(tones):
        s.note(GTR, bar, k * 2, p, 2, vel - (6 if k % 2 else 0))


def drums(bar, full=False):
    s.note(DR, bar, 0, 64, 1, 78)          # low conga
    for st in (4, 8):
        s.note(DR, bar, st, 54, 1, 44)     # tambourine
    if full:
        for st in (2, 6, 10):
            s.note(DR, bar, st, 70, 1, 30) # shaker on the off-beats


A = [  # (pitch, 16ths)
    [("D5", 4), ("B4", 2), ("D5", 2), ("G5", 4)],
    [("F5", 2), ("E5", 2), ("D5", 4), ("B4", 4)],
    [("C5", 4), ("A4", 2), ("C5", 2), ("F5", 4)],
    [("D5", 8), (0, 4)],
    [("E5", 4), ("G5", 2), ("E5", 2), ("C5", 4)],
    [("D5", 2), ("E5", 2), ("D5", 2), ("B4", 2), ("G4", 4)],
    [("A4", 4), ("B4", 2), ("C5", 2), ("D5", 2), ("A5", 2)],
    [("G5", 8), (0, 4)],
]
A_CH = ["G", "G", "F", "G", "C", "G", "D", "G"]
A_HARM = [  # fiddle a third/sixth below, moving with the tune
    [("B4", 4), ("G4", 2), ("B4", 2), ("D5", 4)],
    [("D5", 2), ("C5", 2), ("B4", 4), ("G4", 4)],
    [("A4", 4), ("F4", 2), ("A4", 2), ("C5", 4)],
    [("B4", 8), (0, 4)],
    [("C5", 4), ("E5", 2), ("C5", 2), ("G4", 4)],
    [("B4", 2), ("C5", 2), ("B4", 2), ("G4", 2), ("D4", 4)],
    [("F#4", 4), ("G4", 2), ("A4", 2), ("A4", 2), ("F#5", 2)],
    [("D5", 8), (0, 4)],
]
B = [
    [("B4", 6), ("E5", 2), ("G5", 4)],
    [("G5", 4), ("E5", 4), ("C5", 4)],
    [("A4", 4), ("C5", 2), ("E5", 2), ("A5", 4)],
    [("F#5", 8), ("D#5", 4)],
    [("E5", 4), ("G5", 4), ("B5", 4)],
    [("A5", 4), ("G5", 2), ("E5", 2), ("C5", 4)],
    [("D5", 4), ("F#5", 4), ("A5", 4)],
    [("F#5", 6), ("E5", 2), ("D5", 4)],
]
B_CH = ["Em", "C", "Am", "B", "Em", "C", "D", "D"]
B_ANSWER = [  # the flute answers each fiddle phrase from a distance
    [], [(0, 8), ("E5", 2), ("G5", 2)], [], [(0, 8), ("B4", 2), ("D#5", 2)],
    [], [(0, 8), ("E5", 2), ("C5", 2)], [], [(0, 8), ("C5", 2), ("C5", 2)],
]
# the last A gets little turns, like someone humming along
A_ORN = list(A)
A_ORN[3] = [("D5", 4), ("E5", 1), ("D5", 1), ("C5", 1), ("D5", 1), (0, 4)]
A_ORN[7] = [("G5", 6), ("A5", 1), ("G5", 1), ("F#5", 1), ("G5", 3)]

bar = 0
for i, c in enumerate(A_CH):                  # A: flute alone
    s.seq(FLUTE, bar + i, 0, A[i], 92)
    waltz(bar + i, c)
    drums(bar + i)
bar += 8
for i, c in enumerate(A_CH):                  # A: fiddle harmony
    s.seq(FLUTE, bar + i, 0, A[i], 94)
    s.seq(FIDDLE, bar + i, 0, A_HARM[i], 72)
    waltz(bar + i, c, 74)
    drums(bar + i)
bar += 8
for i, c in enumerate(B_CH):                  # B: the dark outside, fiddle leads
    s.seq(FIDDLE, bar + i, 0, B[i], 96)
    s.seq(FLUTE, bar + i, 0, B_ANSWER[i], 66)
    arp(bar + i, c)
    if i % 2 == 0:
        s.note(DR, bar + i, 0, 64, 1, 66)
bar += 8
for i, c in enumerate(A_CH):                  # A: home again, everyone
    s.seq(FLUTE, bar + i, 0, A_ORN[i], 100)
    s.seq(FIDDLE, bar + i, 0, A_HARM[i], 76)
    waltz(bar + i, c, 78)
    drums(bar + i, full=True)
bar += 8

s.write(os.path.join(os.path.dirname(os.path.abspath(__file__)), "rg_ashford.mid"), bar)
