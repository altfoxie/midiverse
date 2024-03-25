# MIDIverse

MIDI files player for [Pianoverse](https://pianoverse.net). Uses [nodi](https://github.com/insomnimus/nodi) powered by [midly](https://crates.io/crates/midly) to decode and play MIDI files.

## Usage

```bash
Usage: midiverse [OPTIONS] <input>

Arguments:
  <input>  MIDI file to play

Options:
  -r, --room <ROOM>    Room name to join/create [default: midiverse]
  -p, --private        Create a private room
  -t, --transpose <N>  Transpose MIDI notes for N semitones
  -h, --help           Print help
```
