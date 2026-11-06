# Client startup patch (Main.swf)

The original `Main.swf` parses all 431 XML config files (72.8 MB) and builds every lookup table in one frame before it connects. Flash Player did that quickly; Ruffle (WebAssembly) did not: about 9 minutes in headless Chromium, so the browser looked frozen at the loading bar.

`build.py` rewrites 7 decompiled AS3 classes so startup only does the work the first screen needs. Game behavior does not change, only when the work happens:

| Class | Change |
| :--- | :--- |
| `XmlManager`, `XmlReader` | Each of the 297 config fields parses its XML file on first read, not at startup |
| `SkillAndBuffXMLManager` | `skill.xml` (9 MB) is indexed as text; a `<skill/>` is parsed when it is looked up. `children()` hoisted out of the buff loop (it was O(n^2)) |
| `HeroDataUtil` | Monster templates (`monster_teml_copyNeedNot.xml`, 13.6 MB) are merged per `htid` on first lookup. `children()` hoisted out of two loops |
| `PrestigeHerosManager` | Tavern heroes (89 `HeroDetailData` objects) are built on first use |
| `main` | Calls the text-index versions above |
| `MonsterXmlLoader` | No longer appends to `monstersTemplete`, which nothing reads after startup (appending forced the 13.6 MB parse) |

Measured on this machine (headless Chromium, Playwright, Ruffle `0.8.0-nightly.2026.10.22`): the client reaches the server 5.0 to 7.2 s after the game page opens (`web/e2e/game-load.spec.ts`, 4 runs). Before the patch: about 9 minutes.

## Rebuild

Needs Java and JPEXS FFDec (`ffdec-cli.jar`).

1. Keep the original: `client-assets/T9NuoYa/Main.orig.swf`.
2. Decompile it: `java -jar ffdec-cli.jar -export script <out> client-assets/T9NuoYa/Main.orig.swf`.
3. Generate the patched classes: `python -I tools/client-patch/build.py <out>/scripts <patch dir>` (add `trace` as a third argument to log per-step timings to the browser console).
4. Compile them in: `java -jar ffdec-cli.jar -replace client-assets/T9NuoYa/Main.orig.swf client-assets/T9NuoYa/Main.swf <class> <file> ...` with the pairs from `<patch dir>/replace_args.txt` (one per line, class then file).
