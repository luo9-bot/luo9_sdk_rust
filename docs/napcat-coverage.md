# NapCat / OneBot message coverage

This matrix describes what the Rust SDK can model or encode. It is not a claim that every segment/action is supported by every NapCat version: NapCat extensions vary by release and the host must pass the structured data through.

## Message segments

| Case | SDK behavior | Test/limitation |
|---|---|---|
| Standard `text`, `at`, `image`, `reply` segments | `MessageSegment` stores `type` and arbitrary JSON `data`; convenience constructors exist for these common segments | Covered by serde representation and builder |
| Other OneBot v11 segment types (for example face, record, video, json, xml, node) | Can be represented without adding enum variants via `MessageSegment::new(type, data)` | Exact field shape remains NapCat/version-specific |
| Unknown/future NapCat segment types | Type and nested data are preserved as JSON | Covered by `parses_and_preserves_unknown_segment` |
| CQ-code send path | Existing `MsgBuilder` accepts arbitrary segments through `segment` / `raw_segment` | Encodes segment data as CQ code; host transport remains string-based |
| Structured inbound segment array | `MessagePayload.message_segments` can deserialize a segment array when the host includes it | **Requires host-side serialization change**; current host currently serializes the raw CQ/string field and does not populate this field |
| Original upstream event / unknown top-level fields | Optional `raw_event` plus flattened `extra` can retain supplied unknown fields | **Requires host to include original event or forward extra fields**; SDK cannot reconstruct fields discarded upstream |

## Events and actions

This SDK update does not declare all NapCat events or all OneBot actions implemented. The SDK models its existing bus event envelope and preserves unmodelled top-level fields supplied by the host. The host still needs to forward raw event JSON and define an explicit policy for unknown `post_type` values. Likewise, generic segment encoding does not add missing action APIs such as group-member management, file APIs, request approval, or account/profile actions.

## Integration and conformance checklist (P3)

Run these checks against the exact deployed NapCat version and capture sanitized fixtures:

- [ ] Private and group `message` events containing text only
- [ ] Mixed segment arrays: text + at + image + reply
- [ ] Media/structured segments: face, record, video, json, xml, forward/node (where supported)
- [ ] Unknown segment type with nested data survives parse → serialize unchanged
- [ ] Message, notice, request, meta_event, message_sent, and unknown post_type fixtures
- [ ] NapCat extension notices, including emoji-like and notify subtypes
- [ ] Unknown top-level event fields survive host → bus → SDK
- [ ] Send round-trip for supported actions, including escaping and CQ code behavior

A passing SDK unit test only verifies local serialization behavior. Full compatibility requires host integration tests and a versioned NapCat fixture set.
