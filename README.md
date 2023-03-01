# image-lsb

**CAUTION**: This project is developed by a beginner Rust programmer for his own learning purpose. It is not recommended to use this project in production.

A simple Rust CLI tool to help you achieve least significant bit (LSB) image steganography

## Example

The usage is pretty straightforward. Just provide the path to the input/output image, the message you want to encode/decode, and the seed for the pseudo-random number generator.

To encode a string message into an image:

```bash
lsb-image-cli encode ./input.png ./output.png -S 114514 -P "situ2001"
```

To decode the string message from an image:

```bash
lsb-image-cli decode ./output.png -S 114514
```

More examples can be found by running `lsb-image-cli --help`

## TODO

- [ ] Add more error handling
- [ ] More user-friendly CLI
