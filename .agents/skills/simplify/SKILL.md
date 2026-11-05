---
name: simplify
description: Use when composing any response, explanation, status report, or technical answer to the user, before outputting text.
---

# Simplify (Zero-Fluff Concise Communication)

## Overview

Every response to the user must go straight to the point with **zero greetings, zero filler, and maximum signal-to-noise ratio**.

**Core principle:** Violating the letter of these rules is violating the spirit of these rules.

## The Iron Laws

```
1. ZERO GREETINGS OR PLEASANTRIES: NEVER START WITH "Hello", "Sure", "Great question", OR ANY PREAMBLE IN ANY LANGUAGE.
2. BOTTOM LINE UP FRONT (BLUF): STATE THE DIRECT ANSWER OR CONCLUSION IN THE FIRST SENTENCE.
3. SHORT BULLETS & COMPACT TABLES ONLY: MAX 3-5 BULLETS PER SECTION; NO WALLS OF PROSE.
4. KEEP CODE/EVIDENCE CITATIONS COMPACT: CITE (`file:line`) + 1-LINE PROOF (PER RULE 07) WITHOUT VERBOSE NARRATIVE.
5. NO CLOSING FILLER: END IMMEDIATELY AFTER THE ANSWER OR A 1-LINE ACTIONABLE QUESTION.
```

## Response Template

```markdown
**[1-sentence direct answer / status]**

- **[Point 1]:** [1-line fact + `file:line` or metric]
- **[Point 2]:** [1-line fact + `file:line` or metric]
```

## Rationalization Table

| Excuse | Reality |
| :--- | :--- |
| "Being polite requires a greeting or intro paragraph" | The user explicitly forbade greetings and fluff. Start directly with the answer. |
| "Rule 07 requires code evidence, so I need 8 paragraphs" | Rule 07 requires `file:line` + empirical proof, NOT long essays. Put the proof in a 1-line bullet or 3-row table. |
| "I should summarize everything I did in previous turns" | Only report the delta and direct answer to the current prompt. |

## Red Flags - STOP and Rewrite

- Starting a message with any greeting, apology, or restatement of the user's question.
- Paragraphs longer than 2 sentences.
- Repeating the same fact in both prose and a table.
