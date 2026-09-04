#!/bin/bash
# Converts the test image to every supported format and checks that the
# resulting files exist and start with the expected magic bytes.

set -u

status_code=0

check_magic() {
  file="$1"
  expected="$2"
  if [ ! -s "$file" ]; then
    echo "$file was not created"
    status_code=1
    return
  fi
  actual=$(head -c "${#expected}" "$file" | xxd -p | tr -d '\n')
  actual=${actual:0:${#expected}}
  if [ "$expected" != "$actual" ]; then
    echo "$file has an unexpected header. Expected: ${expected}, Actual: ${actual}"
    status_code=1
  fi
}

heif-convert image.heic -f jpg -q 90
heif-convert *.heic -f jpg -q 90 -o image-wildcard
heif-convert image.heic -f png
heif-convert image.heic -f webp
heif-convert image.heic -f gif
heif-convert image.heic -f tiff
heif-convert image.heic -f bmp
heif-convert image.heic -f ico

check_magic image.jpg "ffd8ff"
check_magic image-wildcard.jpg "ffd8ff"
check_magic image.png "89504e470d0a1a0a"
check_magic image.webp "52494646"
check_magic image.gif "474946"
check_magic image.bmp "424d"
check_magic image.ico "00000100"

if [ ! -s image.tiff ]; then
  echo "image.tiff was not created"
  status_code=1
fi

# Invalid input must fail with the argument parser exit code
if heif-convert missing.heic 2>/dev/null; then
  echo "Converting a missing file should have failed"
  status_code=1
fi

exit $status_code
