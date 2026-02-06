# Free Game Assets Research

Research on free/open-licensed assets for a Minecraft-style voxel game built with Bevy/Rust.

---

## 1. Texture Packs (16x16 Block Textures)

### Primary Recommendation: ProgrammerArt

- **URL**: https://github.com/deathcap/ProgrammerArt
- **License**: CC BY 4.0 (Creative Commons Attribution 4.0 International)
- **Format**: 16x16 PNG textures, packaged as Minecraft-compatible resource packs
- **Coverage**: All block and item textures for Minecraft up to 1.9 (grass, dirt, stone, wood, ores, etc.)
- **Why**: Purpose-built as an open-source replacement for Minecraft textures. All artwork is original (not derived from Mojang's art). Permissive license allows any use with attribution. Active GitHub repo with tagged releases.
- **Integration**: Extract individual block PNGs from the `assets/minecraft/textures/blocks/` directory, then stitch them into a texture atlas for the GPU.

### Alternative: Luanti (Minetest) Game Textures

- **URL**: https://github.com/luanti-org/minetest_game/tree/master/mods/default/textures
- **License**: CC BY-SA 3.0 / CC BY-SA 4.0 (mixed, per-texture)
- **Format**: 16x16 PNG
- **Coverage**: Core block types (dirt, stone, wood, sand, water, leaves, ores, etc.)
- **Why**: Battle-tested in a real open-source voxel game. Good art quality. ShareAlike clause means derivative works must use same license.

### Alternative: REFI Textures (for Luanti)

- **URL**: https://github.com/MysticTempest/REFI_Textures
- **License**: CC BY-SA 4.0
- **Format**: 16x16 PNG
- **Coverage**: Comprehensive coverage of Luanti/Minetest blocks including Mineclonia support
- **Why**: Higher quality pixel art than default Minetest textures. Actively maintained.

### Texture Atlas Approach

For a Bevy/wgpu voxel engine, all block textures should be combined into a single **texture atlas** (sprite sheet):

- Use a grid layout (e.g., 16x16 tiles in a 256x256 atlas, or larger)
- Each block face maps to UV coordinates within the atlas
- This allows rendering entire chunks with a single draw call and single texture bind
- Add 1-2px padding between tiles to prevent texture bleeding at mip levels
- Reference: https://0fps.net/2013/07/09/texture-atlases-wrapping-and-mip-mapping/

Build-time script approach: write a small Rust script or use an image crate to stitch individual PNGs into `atlas.png` and generate a mapping JSON (block_name -> atlas UV coordinates).

---

## 2. Sound Effects

### Block Breaking / Placing / Interaction

**75 CC0 Breaking/Falling/Hit SFX** (Primary)
- **URL**: https://opengameart.org/content/75-cc0-breaking-falling-hit-sfx
- **License**: CC0 (Public Domain)
- **Format**: ZIP archive, 1.6 MB, WAV files
- **Contents**: 75 sounds covering wood, metal, glass, rock, stone cracking, impacts, destruction
- **Use for**: Block breaking, block placing, item drops

**100 CC0 SFX #2**
- **URL**: https://opengameart.org/content/100-cc0-sfx-2
- **License**: CC0
- **Format**: WAV/OGG
- **Contents**: Air flowing, door, footsteps, glass, hit, item, ambient loops, metal hits, stones, switch, thunder, wood sounds
- **Use for**: General gameplay sounds, ambient loops, UI feedback

**Pixabay Sound Effects**
- **URL**: https://pixabay.com/sound-effects/search/block/
- **License**: Pixabay License (free for commercial use, no attribution required)
- **Format**: MP3
- **Use for**: Supplementary block/breaking sounds as needed

### Footstep Sounds

**Footsteps on Different Surfaces** (Primary)
- **URL**: https://opengameart.org/content/footsteps-on-different-surfaces
- **License**: CC-BY 3.0
- **Format**: WAV/OGG
- **Surfaces**: Concrete/hard floor, grass/dirt/leaves, gravel/rubble, metal, polished floor, water, wood
- **Use for**: Player footstep sounds varying by block type walked on

**Fantozzi's Footsteps (Grass/Sand & Stone)**
- **URL**: https://opengameart.org/content/fantozzis-footsteps-grasssand-stone
- **License**: CC0
- **Format**: WAV
- **Contents**: 12 single-step sounds for grass/sand and stone surfaces
- **Use for**: Simpler footstep system with fewer surface types

**Foot Walking Steps on Stone, Water, Snow, Wood, Dirt**
- **URL**: https://opengameart.org/content/foot-walking-step-sounds-on-stone-water-snow-wood-and-dirt
- **License**: CC0 (based on pdsounds.org)
- **Surfaces**: Stone, water, snow, wood, dirt
- **Use for**: Additional surface variety

### Ambient Sounds

**CC0 Sound Effects Collection**
- **URL**: https://opengameart.org/content/cc0-sound-effects
- **License**: CC0
- **Contents**: Wind, water, nature ambient loops
- **Use for**: Background environmental ambience (wind in caves, water near rivers, birds in forests)

---

## 3. Music

### Primary: CC0 Calm/Relaxing Music Collection

- **URL**: https://opengameart.org/content/cc0-calm-relaxing-music
- **License**: CC0 (Public Domain)
- **Format**: MP3 and OGG
- **Contents**: 80+ tracks curated by josepharaoh99, including:
  - Ambient piano pieces
  - Medieval-themed tracks (The Bard's Tale, The Old Tower Inn, Harvest Season)
  - Calm exploration music
  - Synthwave ambient tracks
  - RPG town themes
- **Why**: Large collection, all CC0, variety of moods from calm exploration to gentle adventure. Pick 5-10 tracks that fit the Minecraft-like vibe.

### Supplementary Ambient Tracks

**November Snow**
- **URL**: https://opengameart.org/content/november-snow
- **License**: CC0
- **Style**: Ethereal, calm, downtempo ambient. From the Pixelsphere game.
- **Perfect for**: Overworld exploration, nighttime

**CC0 Music Collection**
- **URL**: https://opengameart.org/content/cc0-music-0
- **License**: CC0
- **Contents**: Various atmospheric and electronic ambient tracks
- **Includes**: "The Beach Where Dreams Die", "On Patrol", atmospheric electronic tunes

**CC0 Fantasy Music & Sounds**
- **URL**: https://opengameart.org/content/cc0-fantasy-music-sounds
- **License**: CC0
- **Style**: Fantasy-themed music suitable for medieval/adventure setting

### Recommended Track Selection Strategy

For a Minecraft-like game, select tracks that are:
1. Minimal and ambient (sparse piano, soft pads)
2. Non-intrusive during gameplay
3. Varied enough for different biomes/times of day
4. Loopable or long enough (2+ minutes) to avoid repetition

Aim for 5-8 tracks total: 3-4 overworld day, 2 night/underground, 1 menu theme.

---

## 4. Font

### Primary Recommendation: monogram

- **URL**: https://datagoblin.itch.io/monogram
- **License**: CC0 (Public Domain)
- **Format**: TTF, OTF, FNT, and bitmap variants
- **Size**: ~10 KB for minimal variant
- **Characters**: 104+ characters, monospaced
- **Why**: Clean, legible pixel font perfect for game HUD. Extremely small file size. CC0 means no attribution needed. Has italic subfamily. Popular in indie game dev community.
- **Integration**: Load TTF directly with Bevy's text rendering, or use the bitmap font version for pixel-perfect rendering at fixed sizes.

### Alternatives

**Good Neighbors**
- **URL**: https://opengameart.org/content/good-neighbors-pixel-font
- **License**: CC0
- **Style**: Happy, clean pixel bitmap font designed for crisp scaling

**Pixeldroid Fonts (Console / Menu)**
- **URL**: https://github.com/pixeldroid/fonts
- **License**: SIL Open Font License (OFL)
- **Format**: OTF, TTF, FNT
- **Grid**: 7x7 pixel grid
- **Variants**: "Console" (tiny, 5px upper / 4px lower), "Menu" (game menu style)
- **Why**: Multiple weights/styles available. OFL is very permissive.

---

## 5. Skybox

### Primary Recommendation: Cloudy Skyboxes by Screaming Brain Studios

- **URL (OpenGameArt)**: https://opengameart.org/content/cloudy-skyboxes-0
- **URL (itch.io)**: https://screamingbrainstudios.itch.io/cloudy-skyboxes-pack
- **License**: CC0 (Public Domain)
- **Format**: PNG cubemaps and equirectangular panoramas
- **Contents**: 25 unique skyboxes x 2 projection styles = 50 total skybox images
  - Bright sunny days
  - Sunrises/sunsets
  - Night skies
  - Overcast/cloudy
- **Why**: High quality, procedurally generated. Both cubemap (6-face) and panorama formats available. CC0 license. Covers day/night cycle variations.

### Alternative: Procedural Sky (Recommended for Final Implementation)

For a Minecraft-like game, a **procedural sky shader** is ultimately better than static skyboxes because:
- Smooth day/night transitions
- Dynamic sun/moon positioning
- Color gradients that change with time
- Fog blending at the horizon

**Approach**: Use skybox textures for initial prototyping, then implement a procedural sky shader in wgpu/Bevy:
1. Fragment shader with sky gradient based on sun direction
2. Simple cloud noise layer
3. Star field at night (dot pattern)
4. Sun/moon disc rendering

### Alternative Skyboxes

**Retro Skyboxes Pack**
- **URL**: https://opengameart.org/content/retro-skyboxes-pack
- **License**: CC0
- **Format**: 512x512 PNG + DDS cubemaps
- **Contents**: 11 skies, 19 skyboxes total in retro style

**Free Stylized Skyboxes**
- **URL**: https://freestylized.com/all-skybox/
- **License**: Free for commercial and non-commercial use
- **Styles**: Anime/Ghibli, cloudy, evening/dusk, golden hour, night, sunny

---

## 6. Integration Summary

### Asset Pipeline Plan

```
assets/
  textures/
    blocks/           # Individual 16x16 PNGs from ProgrammerArt
    atlas.png         # Generated texture atlas (build step)
    atlas.json        # UV coordinate mapping
  sounds/
    blocks/
      break/          # Breaking sounds (stone, wood, dirt, glass, etc.)
      place/          # Placing sounds
      dig/            # Digging sounds
    footsteps/
      grass/          # Per-surface footstep sets
      stone/
      wood/
      dirt/
      sand/
    ambient/          # Environmental loops (wind, water, birds)
    ui/               # UI click, inventory sounds
  music/
    overworld_day_1.ogg
    overworld_day_2.ogg
    overworld_night_1.ogg
    underground_1.ogg
    menu.ogg
  fonts/
    monogram.ttf
  skybox/
    day/              # 6 cubemap faces
    sunset/
    night/
```

### License Compliance

| Asset | License | Attribution Required? |
|-------|---------|----------------------|
| ProgrammerArt textures | CC BY 4.0 | Yes - credit deathcap/ProgrammerArt |
| Block breaking SFX | CC0 | No |
| Footsteps (surfaces) | CC-BY 3.0 | Yes - credit author |
| Footsteps (Fantozzi) | CC0 | No |
| Walking steps | CC0 | No |
| Music (calm collection) | CC0 | No |
| monogram font | CC0 | No |
| Cloudy Skyboxes | CC0 | No |

**Recommendation**: Keep an `ATTRIBUTION.md` file in the project crediting all asset sources, even CC0 ones (as a courtesy). Required for CC-BY assets.

### Audio Format Notes

- Convert all audio to **OGG Vorbis** for game use (good compression, Bevy native support)
- Keep WAV originals in a separate `assets-source/` directory (not shipped)
- Normalize volume levels across all sound effects
- Footstep sounds should be short (< 0.5s), music tracks can be 2-5 minutes

### Texture Format Notes

- All block textures should be 16x16 PNG with transparency support
- Texture atlas should use nearest-neighbor (point) filtering, not bilinear, to preserve pixel art crispness
- Use sRGB color space for correct gamma
- Atlas size of 256x256 (16x16 grid of 16x16 tiles) supports 256 unique block faces
