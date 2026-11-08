# Conversations, barks and presenters

Moved from the README on 2026-10-31, unchanged, when the README was brought to the [org documentation guide](https://github.com/dark-incorporated/.github/blob/main/DOCUMENTATION.md).

```rust,ignore
app.add_plugins(SpeechBubblesPlugin::<MyCamera>::default())
    .insert_resource(Presenter::DialogBox) // or Presenter::Balloons, the default
    .insert_resource(my_dialog_skin)       // a DialogSkin built from your theme
    .insert_resource(my_script);           // a DialogueScript, validated

// Balloons need a font (yours to load) and a quad:
commands.insert_resource(BubbleAssets {
    quad: meshes.add(Rectangle::new(1.0, 1.0)),
    font: FontArc::try_from_vec(std::fs::read("assets/fonts/Something.ttf")?)?,
});

speakers.0.insert("keeper".into(), Speaker { entity: Some(keeper), name: "The Keeper".into(), portrait: None });
starts.write(StartConversation { id: "gate".into(), ..default() });
barks.write(Bark { speaker: "keeper".into(), kind: BubbleKind::Speech, emotion: Emotion::Neutral, text: "Mind the step.".into() });
```

While a modal conversation runs the resource `ConversationLock` exists: pause on it, and yield your own click handling to it. A script looks like this (RON; any serde format works):

```ron
(conversations: {
  "gate": (start: "greet", nodes: {
    "greet": Line(speaker: "keeper", text: "The gate has held three nights.", next: Some("ask")),
    "ask": Choice(speaker: "keeper", prompt: "Take the watch?", options: [
      (text: "I will.", next: "yes"),
      (text: "Ask someone else.", next: "no"),
    ]),
    "yes": Line(speaker: "keeper", text: "Good.", next: None),
    "no": Line(speaker: "keeper", emotion: Sadness, text: "Then I will ask the boy.", next: None),
  }),
})
```

A script, a `Bark` and a `BarkSettings` are refused when they carry a key the schema does not have: `emtion: Fear` on a line is a parse error naming the key, the type and the spellings it could have been (`2:48-2:54: Unexpected field named `emtion` in `Line`, expected one of `speaker`, `kind`, `emotion`, `text`, or `next` instead`, with `ron`), not a balloon quietly drawn in the default tint; likewise a `ttl` on a bark or a `max_live` in bark pacing. `emotion` and `kind` stay optional, so a line that omits them still loads. `validate_script` then checks the graph (a dangling `next`, a missing start node, a choice with no options).

`Presenter::Balloons` puts every line over its speaker's head and auto-advances after a reading-speed dwell; a speaker with no entity (a narrator) is skipped quietly, while a key that is not in `Speakers` at all (a misspelt or not-yet-registered one) is skipped with a `warn!` naming it: once per key for a bark, once per node for a line. `Presenter::DialogBox` shows the screen box and waits for the player: `Space`, `Enter` or a click completes a typing line and then advances it; `1`-`9`, the arrows with `Enter`, or the mouse pick an option. Barks are always world balloons.

## One person, two keys: speaker aliases

When two of your systems address one character by different keys (a game's `person:7` and a simulation's `society:7`), register the character once and alias the other key:

```rust,ignore
speakers.0.insert("person:7".into(), Speaker { entity: Some(mara), name: "Mara".into(), portrait: None });
speakers.alias("society:7", "person:7");
```

Every lookup then reads the target's registration: a bark for `society:7` floats over Mara's head and takes its turn in the one bark queue, a balloon-mode line by `society:7` floats there too, and the dialog box shows Mara's name and portrait. An alias resolves one hop (a chain `a → b → c` reads `b`'s registration), wins over a registration of the alias key itself, and goes silent while its target is absent: `speakers.0.remove("person:7")` drops the aliased barks like an unknown key's until the target is registered again. A bark resolves its alias when it is queued, so re-pointing an alias never moves a bark already waiting. `Speakers::unalias` forgets one.

Without an alias, two keys registered on one entity still show one balloon at a time over that head: a new bark replaces the one over its entity, and the queue's gap spaces them.

## How many presses: progress, cost and skip

While a conversation runs, the resource `ConversationProgress` exists (so `Option<Res<ConversationProgress>>` answers "is a conversation up"). It names the node, the line number, the lines left to the end or the next choice, whether the box is still typing, and `presses_to_close()`: the presses that close it from here, `None` while a choice lies ahead. A typed boxed line costs two presses (one completes it, one advances), a box with `chars_per_second <= 0` one, a timed balloon line none; a choice costs its pick, plus one while a boxed prompt types. The count is for a player who confirms as soon as each node is up, so it is an upper bound: pressing exactly that many times closes the conversation, one fewer does not.

Before a conversation starts, ask the script:

```rust,ignore
let rule = PressRule::new(*presenter, skin.chars_per_second);
match script.cost("greet", rule) {
    Ok(c) => info!("{} lines, at most {} presses, branches: {}", c.lines, c.presses, c.branches),
    Err(e) => warn!("{e}"), // an unknown id, a missing node, or a loop (`CostError::Unbounded`)
}
if script.fits("greet", 3, rule) { starts.write(StartConversation { id: "greet".into(), ..default() }); }
```

With a choice, `lines` and `presses` are each the maximum over every path, so a gate on them is a guarantee.

`SkipConversation` finishes the running conversation without the player: it advances node by node, writing `DialogueNodeEntered` for each as presses would, and ends it, or stops at the next choice and leaves the pick. `ConversationEnded` says what the player was there for on every way out: `nodes_seen` (distinct nodes entered), `presses` (presses the conversation used) and `skipped`.

## A conversation the world does not wait for: `Delivery::Unattended`

```rust,ignore
starts.write(StartConversation { id: "night_falls".into(), delivery: Delivery::Unattended });
```

An unattended conversation is drawn by the same presenter (box or balloons, same typewriter, same skin) but takes no `ConversationLock`, reads and consumes no key or click, and closes itself: each line advances once it has finished typing and has been up for its reading time (`dwell_secs`). Your game keeps its keys and its clock. `ConversationProgress` is present while any conversation is up, `ConversationLock` only while a modal one is, so a HUD that hides behind a card asks the first and a pause asks the second. A conversation with a choice (nobody is there to pick) or a loop (it would never close) is refused at the start with a `warn!`. One conversation runs at a time, unattended or not; a start while one runs is ignored, so queue your own hints. `Delivery::Modal` is the default and behaves as before.
