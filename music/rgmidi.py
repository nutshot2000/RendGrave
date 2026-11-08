"""Tiny dependency-free MIDI writer for Rendgrave's soundtrack (rendered with GXSCC).

Steps are a song's own grid (e.g. 16ths, or triplet 8ths), set per song.
Pitches can be MIDI numbers or names like "D5", "F#4", "Bb3".
"""
import struct

NAMES = {"C": 0, "D": 2, "E": 4, "F": 5, "G": 7, "A": 9, "B": 11}


def P(name):
    """'C#5' -> 73. Accepts numbers unchanged; 0 or None means rest."""
    if not name:
        return 0
    if isinstance(name, int):
        return name
    n = NAMES[name[0]]
    i = 1
    while i < len(name) and name[i] in "#b":
        n += 1 if name[i] == "#" else -1
        i += 1
    return n + 12 * (int(name[i:]) + 1)


class Song:
    def __init__(self, bpm, steps_per_bar, steps_per_beat, tpb=480):
        self.tpb = tpb
        self.step = tpb // steps_per_beat
        self.bar_ticks = self.step * steps_per_bar
        self.ev = []  # (tick, order, bytes); order 0 = off/meta first
        self.progs = {}
        self.tempo(0, 0, bpm)

    def t(self, bar, step):
        return int(bar * self.bar_ticks + step * self.step)

    def note(self, ch, bar, step, pitch, steps, vel=100, gap=10):
        p = P(pitch)
        if not p:
            return
        t0 = self.t(bar, step)
        t1 = t0 + max(int(steps * self.step) - gap, 8)
        self.ev.append((t0, 1, bytes([0x90 | ch, p, max(1, min(127, int(vel)))])))
        self.ev.append((t1, 0, bytes([0x80 | ch, p, 0])))

    def chord(self, ch, bar, step, pitches, steps, vel=90):
        for p in pitches:
            self.note(ch, bar, step, p, steps, vel)

    def seq(self, ch, bar, step, notes, vel=100, tr=0):
        """notes: list of (pitch, steps[, vel]); pitch 0 = rest. Returns steps used."""
        s = step
        for n in notes:
            p, d = n[0], n[1]
            v = n[2] if len(n) > 2 else vel
            if p:
                self.note(ch, bar, s, P(p) + tr, d, v)
            s += d
        return s - step

    def tempo(self, bar, step, bpm):
        self.ev.append((self.t(bar, step), 0, b"\xFF\x51\x03" + int(60e6 / bpm).to_bytes(3, "big")))

    def program(self, ch, prog):
        self.progs[ch] = prog

    def write(self, path, end_bar):
        def vlq(n):
            out = [n & 0x7F]
            n >>= 7
            while n:
                out.insert(0, (n & 0x7F) | 0x80)
                n >>= 7
            return bytes(out)

        body = b"".join(b"\x00" + bytes([0xC0 | c, p]) for c, p in self.progs.items())
        last = 0
        end = self.t(end_bar, 0)
        for t, _, d in sorted(self.ev, key=lambda e: (e[0], e[1])):
            if t > end:
                continue
            body += vlq(t - last) + d
            last = t
        body += vlq(end - last) + b"\xFF\x2F\x00"   # end exactly on the loop point
        hdr = b"MThd" + struct.pack(">IHHH", 6, 0, 1, self.tpb)
        open(path, "wb").write(hdr + b"MTrk" + struct.pack(">I", len(body)) + body)
        secs = 0
        print(f"wrote {path}: {end_bar} bars")
