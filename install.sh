#!/usr/bin/env bash
ARCH=$(uname -m)

if [ "$ARCH" != "x86_64" ]; then
    echo "Error: twinkl only supports x86_64 for now. Got: $ARCH"
    exit 1
fi

curl -L https://github.com/kittendevv/twinkl/releases/latest/download/twinkl -o twinkl
chmod +x twinkl
sudo mv twinkl /usr/local/bin/twinkl
echo "twinkl installed! You might need to restart your shell."
