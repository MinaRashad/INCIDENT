# Dialogue JSON Schema

## Structure

The dialogue file is a JSON object where each top-level key is an NPC name, and its value is a map of node names to dialogue nodes.

```
{
    "NPC_Name": {
        "node_name": DialogueNode,
        ...
    }
}
```

---

## DialogueNode

| Field | Type | Description |
|---|---|---|
| `text` | `string \| null` | The message sent to the player. `null` if this node only triggers events with no message. |
| `options` | `Choice[]` | Player choices shown after this node. Empty if no input is needed. |
| `conditions` | `string[]` | Flags that must exist in `history` before this node is processed. |
| `events` | `string[]` | Flags written to `history` when this node is processed. |
| `next` | `string \| null` | The name of the next node to process after this one. `null` if the story ends or waits for player input via options. |

---

## Choice

| Field | Type | Description |
|---|---|---|
| `text` | `string` | The option displayed to the player. |
| `next` | `string` | The node name to jump to when this option is chosen. |
| `event` | `string \| null` | Optional flag written to `history` when this option is chosen. |
| `condition` | `string \| null` | Optional flag that must exist in `history` for this option to be visible to the player. |

---

## Rules

- Every NPC must have a `start` node — this is where the master begins processing
- `next` and `options` are mutually exclusive in practice — if a node has options, `next` should be `null` since the story branches based on player input
- A node with `null` text is still processed normally — conditions are checked, events are fired, and the master advances to `next`
- Conditions are checked against the `history` table in the database at runtime
- Events are written to the `history` table when the node is processed

---

## Example

```json
"Marcus": {
    "start": {
        "text": "Hey, do you want a job?",
        "options": [
            { "text": "Yes.", "next": "job_offer", "event": "expressed_interest", "condition": null },
            { "text": "No.", "next": "end", "event": null, "condition": null }
        ],
        "conditions": [],
        "events": ["met_marcus"],
        "next": null
    }
}
```