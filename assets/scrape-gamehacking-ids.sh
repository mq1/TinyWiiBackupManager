#!/bin/bash
set -e

BROWSER="/Applications/Brave Browser.app/Contents/MacOS/Brave Browser"
UA="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36"

[ -f gamehacking-ids.txt ] && rm gamehacking-ids.txt

for i in {0..70}
do
  echo "Scraping page ${i}"
  "$BROWSER" --headless=new --disable-gpu --virtual-time-budget=10000 --user-agent="$UA" --dump-dom "https://gamehacking.org/system/wii/all/${i}" | grep -A 1 '<td><a href="/game/' >> gamehacking-ids.txt
  sleep 5
done

sed -i '' '/^--$/d' gamehacking-ids.txt
sed -i '' 's/^[[:space:]]*//; s/[[:space:]]*$//' gamehacking-ids.txt
