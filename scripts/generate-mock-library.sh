#!/usr/bin/env bash
set -euo pipefail

OUTPUT_DIR="${1:-test-library}"
echo "===================================================="
echo " Generating Mock Jellyfin Media Library in: $OUTPUT_DIR"
echo "===================================================="

MOVIES_DIR="$OUTPUT_DIR/Movies"
TV_DIR="$OUTPUT_DIR/TV Shows"

mkdir -p "$MOVIES_DIR" "$TV_DIR"

touch_file() {
  mkdir -p "$(dirname "$1")"
  touch "$1"
}

# 1. Clean Jellyfin Movies
make_movie() {
  local title="$1"
  local year="$2"
  local tag="$3"
  local genre="$4"
  local subs="${5:-}"
  local folder="$MOVIES_DIR/$title ($year) $tag"
  mkdir -p "$folder"
  touch "$folder/$title ($year) $tag.mkv"
  touch "$folder/poster.jpg"
  touch "$folder/backdrop.jpg"

  cat > "$folder/movie.nfo" <<EOF
<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<movie>
  <title>$title</title>
  <year>$year</year>
  <genre>$genre</genre>
</movie>
EOF

  if [ -n "$subs" ]; then
    for s in $subs; do
      touch "$folder/$title ($year) $tag.$s.srt"
    done
  fi
}

make_movie "Alien" "1979" "[imdbid-tt0078748]" "Sci-Fi" "en"
make_movie "The Shawshank Redemption" "1994" "[imdbid-tt0111161]" "Drama" "en es"
make_movie "Pulp Fiction" "1994" "[imdbid-tt0110912]" "Crime" "en"
make_movie "The Matrix" "1999" "[imdbid-tt0133093]" "Sci-Fi" "en en.forced"
make_movie "Gladiator" "2000" "[imdbid-tt0172495]" "Action" ""
make_movie "The Lord of the Rings - The Fellowship of the Ring" "2001" "[imdbid-tt0120737]" "Fantasy" "en"
make_movie "Amélie" "2001" "[imdbid-tt0211915]" "Romance" "en fr"
make_movie "The Dark Knight" "2008" "[imdbid-tt0468569]" "Action" "en"
make_movie "Inception" "2010" "[imdbid-tt1375666]" "Sci-Fi" "en"
make_movie "Interstellar" "2014" "[imdbid-tt0816692]" "Sci-Fi" "en de ja"
make_movie "Mad Max - Fury Road" "2015" "[imdbid-tt1392190]" "Action" "en"
make_movie "Arrival" "2016" "[imdbid-tt2543164]" "Sci-Fi" ""
make_movie "Parasite" "2019" "[tmdbid-496243]" "Thriller" "en ko"
make_movie "Dune - Part Two" "2024" "[tmdbid-693134]" "Sci-Fi" "en es"
make_movie "Everything Everywhere All at Once" "2022" "[imdbid-tt6710474]" "Adventure" "en"
make_movie "Oppenheimer" "2023" "[imdbid-tt15398776]" "Biography" "en"
make_movie "Spider-Man - Across the Spider-Verse" "2023" "[imdbid-tt9362722]" "Animation" ""
make_movie "The Godfather" "1972" "[imdbid-tt0068646]" "Crime" "en"
make_movie "2001 - A Space Odyssey" "1968" "[imdbid-tt0062622]" "Sci-Fi" ""
make_movie "No Country for Old Men" "2007" "[imdbid-tt0477348]" "Crime" ""
make_movie "There Will Be Blood" "2007" "[imdbid-tt0469494]" "Drama" "en"
make_movie "WALL-E" "2008" "[imdbid-tt0910970]" "Animation" "en"
make_movie "Whiplash" "2014" "[imdbid-tt2582802]" "Drama" "en"
make_movie "Get Out" "2017" "[imdbid-tt5052448]" "Horror" "en"
make_movie "Spirited Away" "2001" "[tmdbid-129]" "Animation" "ja en"
make_movie "Your Name" "2016" "[tmdbid-372058]" "Animation" "ja en"
make_movie "Akira" "1988" "[imdbid-tt0094625]" "Animation" "ja"
make_movie "Oldboy" "2003" "[imdbid-tt0364569]" "Thriller" ""
make_movie "City of God" "2002" "[imdbid-tt0317248]" "Crime" "pt en"
make_movie "Casablanca" "1942" "[imdbid-tt0034583]" "Romance" "en"

# Multi-Version Blade Runner
BR_DIR="$MOVIES_DIR/Blade Runner (1982) [imdbid-tt0083658]"
mkdir -p "$BR_DIR"
touch "$BR_DIR/Blade Runner (1982) [imdbid-tt0083658] - 1080p Theatrical.mkv"
touch "$BR_DIR/Blade Runner (1982) [imdbid-tt0083658] - 2160p Final Cut.mkv"
touch "$BR_DIR/poster.jpg"
touch "$BR_DIR/backdrop.jpg"

# Messy Scene Movies
SCENE_MOVIES=(
  "the.shining.1980.1080p.BluRay.x264-SPARKS"
  "Fight.Club.1999.2160p.UHD.BluRay.x265-TERMINAL"
  "Se7en.1995.720p.BrRip.x264-YIFY"
  "Memento.2000.1080p.BluRay.H264.AAC-RARBG"
  "the.grand.budapest.hotel.2014.1080p.bluray.x264-sparks"
  "Moonlight.2016.720p.BluRay.x264-SPARKS"
  "la.la.land.2016.2160p.uhd.bluray.x265-terminal"
  "Jojo.Rabbit.2019.1080p.WEB-DL.H264-CMRG"
  "Knives.Out.2019.1080p.BluRay.x264-GECKOS"
  "1917.2019.2160p.UHD.BluRay.x265-TERMINAL"
  "Tenet.2020.1080p.WEB-DL.DD5.1.H.264-EVO"
  "The.Batman.2022.2160p.WEB-DL.DDP5.1.Atmos.HDR.x265"
  "John.Wick.Chapter.4.2023.2160p.WEB-DL.DDP5.1.Atmos.H265-FLUX"
  "Barbie.2023.1080p.BluRay.x264-PiGNUS"
  "Poor.Things.2023.1080p.WEB-DL.DDP5.1.H264-CMRG"
  "The.Zone.of.Interest.2023.1080p.WEB-DL.DD5.1.H264"
  "Anatomy.of.a.Fall.2023.1080p.BluRay.x264-VOA"
  "Past.Lives.2023.720p.WEB-DL.H264-ETHEL"
  "Killers.of.the.Flower.Moon.2023.2160p.WEB-DL.DDP5.1.HDR.H265"
  "Saltburn.2023.1080p.WEB-DL.DDP5.1.H264-CMRG"
)

for sm in "${SCENE_MOVIES[@]}"; do
  mkdir -p "$MOVIES_DIR/$sm"
  touch "$MOVIES_DIR/$sm/$sm.mkv"
done

# TV Shows - Breaking Bad
BB_DIR="$TV_DIR/Breaking Bad (2008) [tvdbid-81189]"
mkdir -p "$BB_DIR"
touch "$BB_DIR/poster.jpg" "$BB_DIR/backdrop.jpg"
cat > "$BB_DIR/tvshow.nfo" <<EOF
<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<tvshow>
  <title>Breaking Bad</title>
  <year>2008</year>
  <uniqueid type="tvdb" default="true">81189</uniqueid>
  <tvdbid>81189</tvdbid>
</tvshow>
EOF

for s in 01 02 03 04 05; do
  mkdir -p "$BB_DIR/Season $s"
  touch "$BB_DIR/season$s-poster.jpg"
  count=13
  [ "$s" = "01" ] && count=7
  [ "$s" = "05" ] && count=16
  for e in $(seq -w 1 $count); do
    if [ "$s" = "05" ] && [ "$e" = "15" ]; then
      touch "$BB_DIR/Season $s/Breaking Bad S05E15-E16 - Granite State & Felina.mkv"
      touch "$BB_DIR/Season $s/Breaking Bad S05E15-E16 - Granite State & Felina.en.srt"
      break
    fi
    touch "$BB_DIR/Season $s/Breaking Bad S${s}E${e} - Episode ${e}.mkv"
    if [ "${e#0}" -le 3 ]; then
      touch "$BB_DIR/Season $s/Breaking Bad S${s}E${e} - Episode ${e}.en.srt"
    fi
  done
done

# Additional TV Shows
for show_info in "Game of Thrones (2011) [tvdbid-121361]:10" "Stranger Things (2016) [tvdbid-305288]:8" "The Mandalorian (2019) [tvdbid-361753]:8" "Arcane (2021) [tvdbid-395256]:9" "Shogun (2024) [tmdbid-126308]:10"; do
  IFS=":" read -r show eps <<< "$show_info"
  s_dir="$TV_DIR/$show/Season 01"
  mkdir -p "$s_dir"
  touch "$TV_DIR/$show/poster.jpg"
  title="${show%% (*}"
  for ep in $(seq -w 1 $eps); do
    touch "$s_dir/$title S01E${ep} - Episode ${ep}.mkv"
  done
done

# Specials
mkdir -p "$TV_DIR/The Mandalorian (2019) [tvdbid-361753]/Season 00"
touch "$TV_DIR/The Mandalorian (2019) [tvdbid-361753]/Season 00/The Mandalorian S00E01 - Disney Gallery.mkv"

# Messy flat scene shows
for mshow in "the.office.us.s01e01.720p.bluray-lol" "better.call.saul.s01e01.1080p.web-dl.dd5.1.h264" "Chernobyl.S01E01.1080p.AMZN.WEB-DL.DDP5.1.H.264-NTb" "black.mirror.s01e01.720p.bluray.x264"; do
  mkdir -p "$TV_DIR/$mshow"
  touch "$TV_DIR/$mshow/$mshow.mkv"
done

echo "✅ Mock Library successfully generated in $OUTPUT_DIR"
