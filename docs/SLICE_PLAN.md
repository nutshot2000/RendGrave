# The Act I slice: making the simulation a choice

**Goal.** Before Act II, make one 20-30 minute stretch of Act I exceptional. Destroying a
particular body part, or changing the environment, should be a *decision* with a visible
payoff, not just how enemies happen to die.

**Rule for every feature.** It must be:
- **readable**: the player can see the weak point or the opportunity;
- **optional**: brute force still works;
- **rewarded**: the clever way is faster, safer or richer.

Each step below is one commit and is playable on its own.

---

## Step 1 - The Hollow King you take apart
The fight moves from "a big health bar" to "a body you dismantle".

**Weak points.** Each has its own small pool. Damage landing on that part of his body fills
it, and the armour visibly cracks and falls away as it does. Every point of damage still
counts against his health too.

| Weak point | Where to hit | Pool | When it breaks |
|---|---|---|---|
| Knee plates | low: legs, kegs, ice spikes, anything at ground level | 7% of his health | The plates fly off and he drops to one knee, stunned. He can no longer do the ground slam and walks slower. |
| Sword arm | the sword hand and forearm, at hip height | 9% | The arm tears off and the sword falls. No more sword swings: he lunges to grab you and throw you instead. |
| Breastplate | the chest | 10% | The plate falls away and exposes a glowing core. He takes +40% damage from everything for the rest of the fight. |

**Hit height decides the target:**
- Weapon hits use where you aim the mouse on his body.
- Spells use the height they hit at: kegs, meteors and ice spikes land on the knees.
- Lightning is drawn to the metal breastplate.

**Second phase at half health, decided by what you broke:**
- *Sword arm intact:* "Rise, my servants" - skeletons and archers, as now.
- *Sword arm severed:* "The archers of the crypt" - a ring of bone archers. You must close in on a King who can only grab.

**Readability:**
- Under his health bar, three small bars: KNEES / SWORD ARM / CHEST, greyed out as "BROKEN".
- The first time you meet him, a tip: aim at his knees, sword hand or chest.
- A big call-out when a part breaks, a stagger, debris, and a distinct sound.

**Done when:**
- Each weak point breaks in testing.
- Each break changes his moves.
- Both second phases happen.
- The boss test can still win with every build.

## Step 2 - Enemies whose anatomy changes the fight
1. **Shield-bearer** (new skeleton in the crypt, bandit at Ravenhold). Blocks blows from the
   front with a wooden or iron shield. Break the shield hand, burn the wooden shield, or get
   behind him. Without the shield he is an ordinary fighter.
2. **Crossbowman: the loaded crossbow.** Sever the firing arm and he drops a loaded crossbow.
   It goes off when anything hits it (a sword, an explosion, fire), firing a bolt along the
   ground through whoever is in line. Enemies can be lined up with it.
3. **Bloater sac.** A visible swollen sac. Puncture it and he leaks acid as he walks: a
   trail you can steer through his friends before he bursts.
4. **Burning panic.** A burning living enemy panics and runs into the nearest ally, setting
   them alight. Sometimes he is more useful alive and on fire.

Each has a visible tell, and a first-time tip the first time you meet it.

## Step 3 - Gear that changes how you fight
Four new uniques, dropped by champions, bosses and secret rooms:

| Item | Effect | Trade-off |
|---|---|---|
| **Splintering Maul** (hammer) | Shattering a frozen enemy drives ice shards through those behind it | Slow |
| **Mudwalker Boots** (new boots slot) | Never slip; +25% damage while standing in oil | Less armour |
| **The Conduit** (sword) | Lightning chains through severed metal armour pieces lying on the ground | |
| **Ember Ring** | Standing near burning ground restores mana | Fire burns you 50% longer |

## Step 4 - Agency from the first minute
- **Starting choice**, at Elder Maren's first conversation. Pick one:
  - *Spark* (Lightning, rank 1);
  - *Oil flasks* (a throwable oil flask with 5 charges that refill in town);
  - *a better weapon* (a magic axe or spear).

  Every path still has a skill point to spend at level 2.
- **Respec** at the shrine by Maren's fire: refund all skill points for a small gold price
  (50 gold x level). Tomes stay learned.
- This replaces unlocking ultimates earlier. Ultimates stay a later-act reward.

## Step 5 - The Ravenhold bridge
A hand-built set piece on the road into Ravenhold:
- Bandits hold a wooden bridge and barricade above a drainage ditch full of oil and water.
- Fight across, burn the barricade, lure them down into the oil and light it, or freeze the
  water and walk across further down.
- A chest behind the barricade.
- All solutions work. It teaches what the systems can do.

## Step 6 - Quiet and consequence
- **Quiet scenes**, each with a few lines of text and a small reward:
  - a burned camp in the Pines with a dying traveller's note;
  - a safe overlook in the Fen;
  - a trapped villager in the Graveyard you can free.
- **Ashford changes** as you win:
  - after the King: a trader returns and the graveyard gate is repaired;
  - after the Chief: the town is lit up, there's a new stall, and different things to say.

---

**After the slice:**
- Play it by hand.
- Tune it from feedback.
- Then start Act II.
