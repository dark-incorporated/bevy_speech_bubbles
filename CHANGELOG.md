# Changelog

All notable changes to this crate are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the crate follows semantic versioning
(pre-1.0: a breaking change bumps the minor version).

## [Unreleased]

Meets the family's machine plate (`dark-incorporated/.github/docs/STYLE.md`):

### Added

- `SpeechBubblesSystems`: every system the plugin adds is in it (`DialogInputSet` is configured into it in `PreUpdate`, the cosmetic systems are in it in `Update`).
- `StartsConversation(pub String)`, an authoring component: the conversation a piece starts. `Reflect`, `Default` and serde, registered by the plugin; nothing in the crate acts on it, the host triggers `StartConversation` for it.
- `BarkSettings` is a settings resource: `Reflect` with `reflect(Resource, Default, Serialize, Deserialize)`, `#[serde(default)]`, registered by the plugin. `Bubble`, `BubblePx`, `BubbleOffsetPx` and `BubbleTtl` derive `Reflect` and are registered.
- `tests/machine_plate.rs`.

## [0.3.0] - 2026-11-06

### Changed

- **Breaking:** `Speakers` gains a private alias table, so it is `Speakers(pub HashMap<String, Speaker>, _)` and can no longer be built as `Speakers(map)`; build it with `Speakers::default()` and fill `.0`, which is unchanged.
- **Breaking:** `ConversationEnded` gains `nodes_seen` (distinct nodes entered), `presses` (presses the conversation used) and `skipped`, written on every way out, so a host learns what was read and can tell a skipped conversation from a read one (#7). It now derives `PartialEq` and `Eq`.
- **Breaking:** `StartConversation` gains `delivery: Delivery` and derives `Default`; write `StartConversation { id, ..default() }` for today's modal conversation (#8).
- The README follows the dark-incorporated org's documentation guide: the org banner, badges, a Status and a Why line, a quick start that is compiled as a doctest, a still image, and "What you can do" bullets linking `docs/guide/`, where every section it used to carry now lives unchanged. Image and file links are absolute URLs, so docs.rs shows them.

### Added

- `Delivery::Unattended` on `StartConversation`: the conversation takes no `ConversationLock`, reads and consumes no key or click, and closes itself, each line advancing once typed and up for its `dwell_secs`, in the box or in balloons. A conversation with a choice or a loop is refused at the start. `ConversationProgress` (with its new `delivery` field) answers "is a conversation up", `ConversationLock` "is it modal" (#8).
- `Speakers::alias(alias, target)`, `Speakers::unalias`, `Speakers::get` and `Speakers::resolve_key`: two speaker keys that are one character (a game's `person:7`, a simulation's `society:7`) share the target's head, name, portrait and bark turn. One hop, the alias wins over the alias key's own registration, a missing target silences it, and a bark resolves its alias when it is queued (#5).
- `ConversationProgress`, a resource present exactly while a conversation runs: its node, `line`, `lines_left` (to the end or the next choice), `typing`, and `presses_to_close()`, the presses that close it from here (a typed boxed line costs two), `None` while a choice lies ahead (#6).
- `SkipConversation`: finishes the running conversation without the player, entering each node as presses would (`DialogueNodeEntered` for each), ending it once or stopping at the next choice (#6).
- `DialogueScript::cost(id, PressRule)` and `fits(id, presses, PressRule)`: a conversation's lines and presses from the script alone, the maximum over its branches, `CostError` for an unknown id, a missing node or a loop; `PressRule::new(presenter, chars_per_second)` is the runtime's own count (#7).
- `examples/dialog_box.rs`: `BUBBLES_SHOT=<file.png>` (with `BUBBLES_FRAMES`, default 30) renders a still
  offscreen on a fixed clock with every line shown at once, and `BUBBLES_CONVERSATION=wrap` starts a
  conversation whose words are longer than the box.

### Changed

- A new runtime dependency, `unicode-segmentation` 1 (pure Rust, no dependencies of its own), for the
  grapheme clusters a long word is broken between.

### Fixed

- A word wider than a balloon line no longer widens the balloon, and the screen dialog box no longer
  lets it run out of the paper. `wrap_lines` was a greedy wrap on whitespace, so a 60-letter word (a
  long name, a URL, a German compound) went on a line of its own and that line set the balloon's width:
  1269x136 px at `BalloonSkin::crt()` and 506x53 at `ink_on_paper()`, against 568 and 234 px for a
  wrapped 240-character line. A line of Chinese or Japanese, which has no spaces, was one such word
  however short. A word that does not fit a line of its own is now broken after the last grapheme
  cluster that fits (UAX #14's emergency mode; an accent, a flag or an emoji sequence is never split) and
  the next word continues after its last fragment. A word that fits on a line is never broken, so
  ordinary text wraps exactly as before. The dialog box's body text and choice rows ask Bevy for
  `LineBreak::WordOrCharacter`, which is the same rule.

- A misspelt or unknown key in authored data is a parse error, not a value silently left at its
  default. `Node`, `Bark` and `BarkSettings` now refuse unknown fields (`#[serde(deny_unknown_fields)]`),
  as `Choice`, `Conversation` and `DialogueScript` already did. `Line(.., emtion: Fear, ..)` used to
  load as a Neutral balloon and `knid: Thought` as a speech balloon, `ttl` on a bark and `max_live` in
  bark pacing were dropped without a word, and `validate_script` could not see any of it. The error
  names the key, the type and the spellings it could have been, with the line and column (RON). Scripts
  and barks written with the correct keys, and everything this crate serialises, load as before; a host
  whose files carry an unknown key that used to be ignored now gets an error at load time (#12).

- A `Bark`, or a `Presenter::Balloons` line, whose speaker is not a key of `Speakers` is no longer
  dropped in silence. The crate told a registered narrator (`entity: None`, quiet by design) from a key
  nobody registered with one `None`; it now tells them apart and `warn!`s for the second, naming the
  key and the bark's text, or the line's node and conversation. A bark's warning is once per key (a
  per-key set, bounded at 64 keys), not once per call site, so a second misspelt key still speaks up;
  what is shown is unchanged (#11).

## [0.2.2] - 2026-10-31

### Added

- `Bark` and `BarkSettings` are `serde` types, so a game or a tool authors barks and their pacing as
  data (RON) with no second schema (#9).
- `examples/bench.rs`: what the kit costs its caller, one line per case (raster time and bytes per
  balloon, frame cost at 0/50/200 live balloons and on child owners, the worst frame of a bark burst,
  asset counts after 1000 barks, presses per conversation).
- `examples/two_balloons.rs`: the thinker rides a cart as a child entity, its balloons cast no shadow on
  the floor (`NotShadowCaster`, as the crate's own balloons), and `BUBBLES_SHOT=<file.png>` (with
  `BUBBLES_FRAMES`) renders a still offscreen on a fixed clock and exits.

### Fixed

- A speaker that is a child entity gets its balloon over its world position. `track_bubbles` read the
  owner's local `Transform`, so a balloon on a child floated at its parent-relative offset from the
  world origin; it now composes the owner's and its ancestors' `Transform`s in the frame it tracks, so
  it matches the propagated `GlobalTransform` with no frame of lag. The balloon also hides when its
  owner is hidden through a parent, by Bevy's visibility inheritance, and its `Visibility` is written
  only when it changes, so a steady balloon no longer re-runs visibility propagation every frame (#10).
