# Architecture

WIP 

## IPC Protocol (v1)

All messages are little-endian, 32-bit length-prefixed payloads.

### Request Example

```json
{ "v": 1, "prompt": "...", "stream": true }
```

### Streaming Response Example

```json
{ "token": "Par", "eos": false }
```

### Final Response Example

```json
{ "token": null, "eos": true }
```

### Field Types and Semantics

- **v**: Must equal 1 (version number)
- **prompt**: UTF-8 encoded string containing the user's input
- **stream**: Boolean indicating whether to stream the response
- **token**: UTF-8 encoded string containing the generated token, or null when complete
- **eos**: Boolean indicating end-of-stream (true when generation is complete)

Note that future versions will bump "v" and stay backward-compatible via feature flags. 