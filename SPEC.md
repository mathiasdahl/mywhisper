# Specification

## Output channels

Each output channel should be possible to enforce on the command line. Multiple options are allowed.

- stdout (default) - DONE
- Windows clipboard - DONE

### Offer channel options - DONE

When the /y option is not used and after the recording is done, show the transcripted text, and wait for user input and offer the following options:

- s - write to stdout
- c - clipboard
- Esc or q - just quit, output nothing on any channel (but still log everything)

## Logging

- Each recording should be saved to a common folder, name it with a proper timestamp - DONE
- Each transcription (the output text) should be kept alongside the recording in a text file - DONE

