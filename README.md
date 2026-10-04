<div align="center">
  <h1>Warcraft Survivors</h1>
  <p><b>A Vampire Survivors-style roguelite played in the world of World of Warcraft 1.12.1</b></p>
  <p>Built on <a href="https://github.com/samwhosung/benilla">benilla</a>, a from-scratch 1.12.1 client in Rust and <a href="https://bevy.org">Bevy</a></p>
</div>

Pick a class and a hero, pick a battleground, and survive fifteen minutes against an ever-growing
horde of the zone's own creatures. Your spells cast themselves; you only move. Every level offers
three cards: a new spell from your class, a higher rank of one you have, or a passive blessing.
A boss arrives every two and a quarter minutes, and the last of them guards the final stretch.

Everything on screen is the real game, read at runtime from your own 1.12.1 install: the
terrain, the creatures and their animations, the spell visuals, the sounds and the interface art.
This repository ships none of it.

## What you need

- **Your own English World of Warcraft 1.12.1 client (build 5875).** Warcraft Survivors only
  reads it, and never writes to it.
- **Rust**, from [rustup](https://rustup.rs). The repository pins its toolchain in
  `rust-toolchain.toml`, so rustup fetches the right version on the first build.
- **A C compiler**, because the client's Lua is built from source:
  - Windows: the MSVC build tools, which the Rust installer offers to set up.
  - macOS: the Xcode command line tools (`xcode-select --install`).
  - Linux: the ALSA and udev development packages and `pkg-config` (on Debian or Ubuntu,
    `sudo apt install build-essential pkg-config libasound2-dev libudev-dev`).

No server and no account are needed: the mode runs entirely offline.

## Setup

1. Clone this branch:

   ```sh
   git clone -b survivors https://github.com/AdamMcWilliam/warcraft-survivors.git
   cd warcraft-survivors
   ```

2. Point it at your WoW install, either by linking the install folder as `WoW` at the repo root
   (the folder that contains `WoW.exe` and `Data`):

   ```powershell
   # Windows (PowerShell, no admin rights needed)
   New-Item -ItemType Junction -Path WoW -Target "C:\path\to\WoW"
   ```

   ```sh
   # macOS / Linux
   ln -s /path/to/WoW WoW
   ```

   or by setting `WOW_DATA` to the install's `Data` folder each time you run it
   (`$env:WOW_DATA="C:\path\to\WoW\Data"` in PowerShell, `export WOW_DATA=/path/to/WoW/Data`
   elsewhere). The `WoW` link is ignored by git, so it never gets committed.

3. Build and play:

   ```sh
   cargo survivors
   ```

   That is an alias for `cargo run --profile play -p warcraft-survivors`. The first build
   compiles the whole engine and takes several minutes; after that it starts in seconds. The
   built game is `target/play/warcraft-survivors` (`.exe` on Windows), which you can also launch
   directly or pin as a shortcut.

## How to play

| Input | Does |
|---|---|
| `W` `A` `S` `D` or the arrow keys | Move |
| Mouse wheel | Zoom the camera |
| Hover a spell or passive icon (top left) | Its tooltip: rank, range, cooldown, what it does now and what the next rank adds |
| `1` `2` `3` or click | Take a level-up card |
| `Esc` | Pause and resume |
| `Enter` | Start the run from the menu, or return to the menu after one |

- **Classes:** Warrior, Paladin, Hunter, Rogue, Priest, Shaman, Mage, Warlock and Druid, each
  with its own spell pool and the real spell visuals. The heroes on offer are the looks of that
  class's trainers.
- **Experience:** slain enemies drop wisps; walk near them to pull them in. Now and then an enemy
  drops a turkey leg that heals you, and every boss drops one that heals you fully.
- **Pressure:** the horde grows and toughens over the run, and half way between bosses a ring of
  enemies closes in from every side.
- **Victory:** still standing at 15:00.

## Battlegrounds

Each battleground has its own roster that climbs from the zone's weakest creatures to its
deadliest, and six bosses drawn from the zone and its dungeons and raids. Each is a compact
arena: an invisible boundary keeps the fight in view.

| Battleground | Zone | The horde | Final boss |
|---|---|---|---|
| The Barrens | Kalimdor | Plains beasts, quilboar, centaur and harpies | Hezrul Bloodmark |
| The Dark Portal | Blasted Lands | Hyenas, scorpids and the Burning Legion | Lord Kazzak |
| Gates of Ahn'Qiraj | Silithus | The silithid swarm and the Qiraji | Ossirian the Unscarred |
| Fire Plume Ridge | Un'Goro Crater | Dinosaurs, oozes and fire elementals, beside the lava lake | King Mosh |
| Gurubashi Arena | Stranglethorn Vale | Jungle beasts and the Gurubashi trolls | Hakkar |
| Blackrock Mountain | Burning Steppes | The Blackrock orcs, worgs and the black dragonflight | Nefarian |
| Kodo Graveyard | Desolace | Scorpashi, basilisks, demons and dying kodo | Princess Theradras |
| Winterspring | Lake Kel'Theril | Owls, chimaeras, Highborne and the blue dragonflight | Azuregos |
| Mount Hyjal | Kalimdor | The Legion, the Scourge and the dragons of Nightmare | Ysondre |
| Naxxramas | Eastern Plaguelands | The Scourge of Plaguewood, beneath the necropolis | Kel'Thuzad |

Mount Hyjal is unfinished in 1.12 and has no creatures of its own, so its horde is a themed one.

## For developers

The mode lives in [`crates/benilla-app/src/survivors/`](crates/benilla-app/src/survivors/) (the
classes, spells, battlegrounds and their rosters are tables in `data.rs`), and its launcher in
[`crates/warcraft-survivors/`](crates/warcraft-survivors/). It boots the benilla client with no
server: the hero, the horde and the drops are local entities dressed by the engine's own model,
animation and spell-visual systems.

A hands-off autopilot plays a run for testing:

```powershell
$env:WOW_SURVIVORS_AUTO="Mage"         # a class; add :all for its whole spell pool, :late to start deep into a run, :idle to stand still
$env:WOW_SURVIVORS_MAP="Naxxramas"     # a battleground, by index (0-9) or part of its name
$env:WOW_SURVIVORS_SHOTS="C:\shots"    # optional: a screenshot every few seconds
cargo survivors
```

It logs a line every ten seconds (`survivors auto: ...`). The rest of benilla, the complete 1.12.1
client, is still here and still builds: see the
[benilla README](https://github.com/samwhosung/benilla#running-it) and
[`docs/`](docs/).

## Credits and legal

Warcraft Survivors is a fork of [benilla](https://github.com/samwhosung/benilla) by its authors,
which does all the heavy lifting: every file format, the renderer, the animation and spell
systems, and the interface engine.

This is an independent fan project, not affiliated with or endorsed by Blizzard Entertainment.
It ships **no Blizzard content**: no art, models, sounds, maps, MPQ contents or FrameXML. You
provide your own legally obtained 1.12.1 client. World of Warcraft and Warcraft are trademarks of
Blizzard Entertainment, Inc. Vampire Survivors is a trademark of poncle.

The code is licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your option, as
benilla is. The two vendored components under `third_party/`, the kira audio engine and a Lua 5.1
patched to the 1.12 client's dialect, keep their own upstream licenses, alongside each.
