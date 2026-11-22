---
trigger: always_on
description: Strict Jellyfin file/folder naming conventions, provider ID tags, NFO XML schemas, and filesystem safety guardrails.
---

# Jellyfin Media Management & Filesystem Standards

All file scanners, metadata parsers, NFO writers, and batch renamers in this codebase MUST strictly adhere to Jellyfin's official library specifications. Never mix Plex conventions (such as `{imdb-tt...}` curly braces) into Jellyfin paths.

---

## 1. Provider ID Tag Format

Jellyfin uses square brackets with a hyphenated provider prefix in folder and filenames:
- **IMDb:** `[imdbid-tt1234567]` (always include the `tt` prefix)
- **TMDB:** `[tmdbid-12345]`
- **TVDB:** `[tvdbid-12345]`

When multiple IDs are available, prefer `[imdbid-tt...]` or `[tmdbid-...]` for movies and `[tvdbid-...]` or `[tmdbid-...]` for TV series. Only include one primary provider ID tag in the path to keep filenames clean, while storing all secondary IDs inside the `.nfo` file.

---

## 2. Movie Naming & Folder Structure

### Standard Layout
Each movie must reside in its own dedicated folder:
```text
Movies/
└── Movie Name (Year) [imdbid-tt1234567]/
    ├── Movie Name (Year) [imdbid-tt1234567].mkv
    ├── Movie Name (Year) [imdbid-tt1234567].en.srt
    ├── Movie Name (Year) [imdbid-tt1234567].en.forced.srt
    ├── movie.nfo
    ├── poster.jpg
    └── backdrop.jpg
```

### Multi-Version Movies (Multiple Resolutions / Cuts)
Only append ` - Label` when two or more video files for the same movie exist in the same directory. The prefix before ` - ` MUST match the parent folder name exactly:
```text
Movies/
└── Blade Runner (1982) [imdbid-tt0083658]/
    ├── Blade Runner (1982) [imdbid-tt0083658] - 1080p Theatrical.mkv
    ├── Blade Runner (1982) [imdbid-tt0083658] - 2160p Final Cut.mkv
    └── movie.nfo
```

### Multi-Part Movies & Extras
- **Multi-part files:** Suffix with `- part1`, `- part2`, `- cd1`, or `- cd2` (e.g., `Movie Name (Year) - part1.mkv`).
- **Local extras subfolders:** Preserve recognized Jellyfin subfolders untouched during scans/renames: `trailers`, `behind the scenes`, `deleted scenes`, `featurettes`, `interviews`, `scenes`, `shorts`, `extras`.

---

## 3. TV Show Naming & Folder Structure

### Standard Layout
```text
TV Shows/
└── Series Name (Year) [tvdbid-12345]/
    ├── tvshow.nfo
    ├── poster.jpg
    ├── backdrop.jpg
    ├── banner.jpg
    ├── Season 00/
    │   ├── Series Name S00E01 - Special Title.mkv
    │   └── Series Name S00E01 - Special Title.nfo
    └── Season 01/
        ├── season01-poster.jpg
        ├── Series Name S01E01 - Episode Title.mkv
        ├── Series Name S01E01 - Episode Title.en.srt
        └── Series Name S01E01 - Episode Title.nfo
```

### TV Naming Rules
- **Season folders:** Always zero-pad to at least 2 digits (`Season 01`, `Season 02`). Use `Season 00` for Specials.
- **Episode numbers:** Always use `SXXEXX` format (`S01E01`). For shows exceeding 99 episodes in a single season (e.g., anime/daily shows), zero-pad episodes to 3 digits (`S01E105`).
- **Multi-episode files:** Format as `Series Name S01E01-E02 - Episode Title.ext`.
- **Episode Ordering:** Support `Aired`, `DVD`, `Streaming`, and `Absolute` episode orders. When switching order modes, remap `Season XX` and `SXXEXX` numbers to match the selected order while preserving original aired metadata in `<episodedetails>`.

---

## 4. Sidecars, Subtitles & Artwork

When renaming or moving a video file, **all matching sidecar files MUST be moved and renamed atomically** to match the new video stem:
- **Subtitles:** `<VideoStem>.<lang>[.<flag>].<ext>`
  - Use ISO 639-1 (`en`, `es`, `ja`) or ISO 639-2 (`eng`, `spa`, `jpn`) codes.
  - Preserve flags in order: `.forced`, `.sdh`, `.cc`, `.default` (e.g., `Movie (2024).en.forced.srt`).
- **Artwork filenames:**
  - Movie / Series root: `poster.jpg`, `backdrop.jpg` (or `fanart.jpg`), `logo.png`, `banner.jpg`, `thumb.jpg`.
  - Season artwork (placed in series root): `season01-poster.jpg`, `season-specials-poster.jpg`.

---

## 5. Jellyfin NFO XML Specifications

All NFO files must be UTF-8 encoded XML with no BOM. Always write both `<uniqueid type="..." default="...">` (Kodi/Jellyfin standard) and legacy provider tags (`<imdbid>`, `<tmdbid>`, `<tvdbid>`) for maximum compatibility.

### Movie NFO (`movie.nfo`)
```xml
<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<movie>
  <title>Movie Title</title>
  <originaltitle>Original Title</originaltitle>
  <sorttitle>Movie Title</sorttitle>
  <year>2024</year>
  <premiered>2024-05-15</premiered>
  <plot>Full plot synopsis.</plot>
  <tagline>Movie tagline.</tagline>
  <runtime>142</runtime>
  <mpaa>PG-13</mpaa>
  <ratings>
    <rating name="imdb" max="10" default="true">
      <value>8.2</value>
      <votes>154000</votes>
    </rating>
    <rating name="themoviedb" max="10">
      <value>7.9</value>
      <votes>4200</votes>
    </rating>
  </ratings>
  <uniqueid type="imdb" default="true">tt1234567</uniqueid>
  <uniqueid type="tmdb">12345</uniqueid>
  <imdbid>tt1234567</imdbid>
  <tmdbid>12345</tmdbid>
  <genre>Sci-Fi</genre>
  <studio>Warner Bros. Pictures</studio>
  <director>Director Name</director>
  <credits>Writer Name</credits>
  <actor>
    <name>Actor Name</name>
    <role>Character Name</role>
    <order>0</order>
    <thumb>[https://image.tmdb.org/t/p/original/](https://image.tmdb.org/t/p/original/)...</thumb>
  </actor>
</movie>
```

### TV Series NFO (`tvshow.nfo`)
```xml
<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<tvshow>
  <title>Series Title</title>
  <originaltitle>Original Series Title</originaltitle>
  <sorttitle>Series Title</sorttitle>
  <year>2022</year>
  <premiered>2022-09-01</premiered>
  <status>Continuing</status>
  <plot>Series overview.</plot>
  <mpaa>TV-MA</mpaa>
  <displayorder>aired</displayorder>
  <ratings>
    <rating name="imdb" max="10" default="true">
      <value>8.8</value>
      <votes>98000</votes>
    </rating>
  </ratings>
  <uniqueid type="tvdb" default="true">12345</uniqueid>
  <uniqueid type="imdb">tt7654321</uniqueid>
  <uniqueid type="tmdb">67890</uniqueid>
  <tvdbid>12345</tvdbid>
  <imdbid>tt7654321</imdbid>
  <tmdbid>67890</tmdbid>
  <genre>Drama</genre>
  <studio>HBO</studio>
</tvshow>
```

### Episode NFO (`<EpisodeFileName>.nfo`)
```xml
<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<episodedetails>
  <title>Episode Title</title>
  <showtitle>Series Title</showtitle>
  <season>1</season>
  <episode>1</episode>
  <aired>2022-09-01</aired>
  <plot>Episode synopsis.</plot>
  <runtime>58</runtime>
  <ratings>
    <rating name="imdb" max="10" default="true">
      <value>8.6</value>
      <votes>5400</votes>
    </rating>
  </ratings>
  <uniqueid type="tvdb" default="true">987654</uniqueid>
  <uniqueid type="imdb">tt1122334</uniqueid>
  <director>Episode Director</director>
  <credits>Episode Writer</credits>
  <!-- Include these only for Season 00 specials to place them in playback order -->
  <!-- <airsbefore_season>2</airsbefore_season> -->
  <!-- <airsbefore_episode>1</airsbefore_episode> -->
</episodedetails>
```

---

## 6. Path Sanitization & Filesystem Safety Guardrails

1. **Illegal Character Sanitization:** Strip or replace characters invalid on Windows/SMB/macOS/Linux before constructing any path:
   - Replace `:` with ` -` (e.g., `Dune: Part Two` becomes `Dune - Part Two`).
   - Replace `/` and `\` with `-`.
   - Strip `< > " | ? *` and trailing periods/spaces (Windows NTFS forbids trailing dots/spaces in folder names).
2. **Dry-Run First:** Never execute destructive `fs::rename` or `fs::remove` calls directly from a UI button without first generating a `RenamePlan` diff and requiring explicit user confirmation.
3. **Cross-Filesystem Moves:** Handle `EXDEV` (cross-device link errors on network mounts/SMB shares) gracefully by falling back to copy + checksum/size verification + source delete.
4. **Transaction Logging:** Log every executed rename batch (`batch_id`, `old_path`, `new_path`, `timestamp`) to SQLite so every rename operation is reversible via one-click Undo.