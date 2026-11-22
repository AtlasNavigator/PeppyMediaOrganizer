use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

fn touch_file(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    File::create(path).unwrap();
}

fn write_nfo(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut file = File::create(path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
}

fn movie_nfo(title: &str, year: u16, imdb_id: Option<&str>, tmdb_id: Option<u32>, genre: &str) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?>\n<movie>\n");
    xml.push_str(&format!("  <title>{}</title>\n", title));
    xml.push_str(&format!("  <year>{}</year>\n", year));
    if let Some(imdb) = imdb_id {
        xml.push_str(&format!("  <uniqueid type=\"imdb\" default=\"true\">{}</uniqueid>\n", imdb));
        xml.push_str(&format!("  <imdbid>{}</imdbid>\n", imdb));
    }
    if let Some(tmdb) = tmdb_id {
        let is_default = if imdb_id.is_none() { " default=\"true\"" } else { "" };
        xml.push_str(&format!("  <uniqueid type=\"tmdb\"{}>{}</uniqueid>\n", is_default, tmdb));
        xml.push_str(&format!("  <tmdbid>{}</tmdbid>\n", tmdb));
    }
    xml.push_str(&format!("  <genre>{}</genre>\n", genre));
    xml.push_str("</movie>\n");
    xml
}

fn tvshow_nfo(title: &str, year: u16, tvdb_id: Option<u32>, tmdb_id: Option<u32>, imdb_id: Option<&str>, genre: &str) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?>\n<tvshow>\n");
    xml.push_str(&format!("  <title>{}</title>\n", title));
    xml.push_str(&format!("  <year>{}</year>\n", year));
    if let Some(tvdb) = tvdb_id {
        xml.push_str(&format!("  <uniqueid type=\"tvdb\" default=\"true\">{}</uniqueid>\n", tvdb));
        xml.push_str(&format!("  <tvdbid>{}</tvdbid>\n", tvdb));
    }
    if let Some(tmdb) = tmdb_id {
        xml.push_str(&format!("  <uniqueid type=\"tmdb\">{}</uniqueid>\n", tmdb));
        xml.push_str(&format!("  <tmdbid>{}</tmdbid>\n", tmdb));
    }
    if let Some(imdb) = imdb_id {
        xml.push_str(&format!("  <uniqueid type=\"imdb\">{}</uniqueid>\n", imdb));
        xml.push_str(&format!("  <imdbid>{}</imdbid>\n", imdb));
    }
    xml.push_str(&format!("  <genre>{}</genre>\n", genre));
    xml.push_str("</tvshow>\n");
    xml
}

fn episode_nfo(show_title: &str, season: u16, episode: u16, ep_title: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8" standalone="yes"?>
<episodedetails>
  <title>{}</title>
  <showtitle>{}</showtitle>
  <season>{}</season>
  <episode>{}</episode>
</episodedetails>
"#,
        ep_title, show_title, season, episode
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let root_dir = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("test-library")
    };

    println!("====================================================");
    println!(" Generating Jellyfin Mock Library in: {}", root_dir.display());
    println!("====================================================");

    let movies_root = root_dir.join("Movies");
    let tvshows_root = root_dir.join("TV Shows");

    fs::create_dir_all(&movies_root).unwrap();
    fs::create_dir_all(&tvshows_root).unwrap();

    let mut movie_count = 0;
    let mut show_count = 0;
    let mut episode_count = 0;

    // ==========================================
    // 1. CLEAN JELLYFIN MOVIES (30 titles)
    // ==========================================
    let clean_movies: &[(&str, u16, Option<&str>, Option<u32>, &str, bool, &[&str], bool, bool)] = &[
        // (title, year, imdb_id, tmdb_id, genre, has_nfo, subs, has_poster, has_backdrop)
        ("Alien", 1979, Some("tt0078748"), None, "Sci-Fi", true, &["en"], true, false),
        ("The Shawshank Redemption", 1994, Some("tt0111161"), None, "Drama", true, &["en", "es"], true, true),
        ("Pulp Fiction", 1994, Some("tt0110912"), None, "Crime", true, &["en"], true, false),
        ("The Matrix", 1999, Some("tt0133093"), None, "Sci-Fi", true, &["en", "en.forced"], true, true),
        ("Gladiator", 2000, Some("tt0172495"), None, "Action", false, &[], false, false),
        ("The Lord of the Rings - The Fellowship of the Ring", 2001, Some("tt0120737"), None, "Fantasy", true, &["en"], true, true),
        ("Amélie", 2001, Some("tt0211915"), None, "Romance", true, &["en", "fr"], true, false),
        ("The Dark Knight", 2008, Some("tt0468569"), None, "Action", true, &["en"], true, true),
        ("Inception", 2010, Some("tt1375666"), None, "Sci-Fi", true, &["en"], true, false),
        ("Interstellar", 2014, Some("tt0816692"), None, "Sci-Fi", true, &["en", "de", "ja"], true, true),
        ("Mad Max - Fury Road", 2015, Some("tt1392190"), None, "Action", true, &["en"], true, false),
        ("Arrival", 2016, Some("tt2543164"), None, "Sci-Fi", false, &[], false, false),
        ("Parasite", 2019, None, Some(496243), "Thriller", true, &["en", "ko"], true, true),
        ("Dune - Part Two", 2024, None, Some(693134), "Sci-Fi", true, &["en", "es"], true, true),
        ("Everything Everywhere All at Once", 2022, Some("tt6710474"), None, "Adventure", true, &["en"], true, false),
        ("Oppenheimer", 2023, Some("tt15398776"), None, "Biography", true, &["en"], true, true),
        ("Spider-Man - Across the Spider-Verse", 2023, Some("tt9362722"), None, "Animation", false, &[], true, false),
        ("The Godfather", 1972, Some("tt0068646"), None, "Crime", true, &["en"], true, true),
        ("2001 - A Space Odyssey", 1968, Some("tt0062622"), None, "Sci-Fi", true, &[], false, false),
        ("No Country for Old Men", 2007, Some("tt0477348"), None, "Crime", false, &[], false, false),
        ("There Will Be Blood", 2007, Some("tt0469494"), None, "Drama", true, &["en"], true, false),
        ("WALL-E", 2008, Some("tt0910970"), None, "Animation", true, &["en"], true, true),
        ("Whiplash", 2014, Some("tt2582802"), None, "Drama", false, &["en"], false, false),
        ("Get Out", 2017, Some("tt5052448"), None, "Horror", true, &["en"], true, false),
        ("Spirited Away", 2001, None, Some(129), "Animation", true, &["ja", "en"], true, true),
        ("Your Name", 2016, None, Some(372058), "Animation", true, &["ja", "en"], true, false),
        ("Akira", 1988, Some("tt0094625"), None, "Animation", true, &["ja"], true, false),
        ("Oldboy", 2003, Some("tt0364569"), None, "Thriller", false, &[], false, false),
        ("City of God", 2002, Some("tt0317248"), None, "Crime", true, &["pt", "en"], true, true),
        ("Casablanca", 1942, Some("tt0034583"), None, "Romance", true, &["en"], true, false),
    ];

    for &(title, year, imdb, tmdb, genre, has_nfo, subs, has_poster, has_backdrop) in clean_movies {
        let tag = if let Some(id) = imdb {
            format!("[imdbid-{}]", id)
        } else if let Some(id) = tmdb {
            format!("[tmdbid-{}]", id)
        } else {
            String::new()
        };

        let folder_name = format!("{} ({}) {}", title, year, tag).trim().to_string();
        let folder_path = movies_root.join(&folder_name);
        fs::create_dir_all(&folder_path).unwrap();

        let video_stem = folder_name.clone();
        let video_file = folder_path.join(format!("{}.mkv", video_stem));
        touch_file(&video_file);

        if has_nfo {
            let nfo_content = movie_nfo(title, year, imdb, tmdb, genre);
            write_nfo(&folder_path.join("movie.nfo"), &nfo_content);
        }

        if has_poster {
            touch_file(&folder_path.join("poster.jpg"));
        }

        if has_backdrop {
            touch_file(&folder_path.join("backdrop.jpg"));
        }

        for sub_lang in subs {
            let sub_name = format!("{}.{}.srt", video_stem, sub_lang);
            touch_file(&folder_path.join(&sub_name));
        }

        movie_count += 1;
    }

    // Special Multi-Version Movie: Blade Runner (1982) [imdbid-tt0083658]
    {
        let br_folder = movies_root.join("Blade Runner (1982) [imdbid-tt0083658]");
        fs::create_dir_all(&br_folder).unwrap();
        touch_file(&br_folder.join("Blade Runner (1982) [imdbid-tt0083658] - 1080p Theatrical.mkv"));
        touch_file(&br_folder.join("Blade Runner (1982) [imdbid-tt0083658] - 2160p Final Cut.mkv"));
        touch_file(&br_folder.join("poster.jpg"));
        touch_file(&br_folder.join("backdrop.jpg"));
        let nfo = movie_nfo("Blade Runner", 1982, Some("tt0083658"), None, "Sci-Fi");
        write_nfo(&br_folder.join("movie.nfo"), &nfo);
        movie_count += 1;
    }

    // ==========================================
    // 2. MESSY SCENE-RELEASE MOVIES (20 titles)
    // ==========================================
    let scene_movies = &[
        "the.shining.1980.1080p.BluRay.x264-SPARKS",
        "Fight.Club.1999.2160p.UHD.BluRay.x265-TERMINAL",
        "Se7en.1995.720p.BrRip.x264-YIFY",
        "Memento.2000.1080p.BluRay.H264.AAC-RARBG",
        "the.grand.budapest.hotel.2014.1080p.bluray.x264-sparks",
        "Moonlight.2016.720p.BluRay.x264-SPARKS",
        "la.la.land.2016.2160p.uhd.bluray.x265-terminal",
        "Jojo.Rabbit.2019.1080p.WEB-DL.H264-CMRG",
        "Knives.Out.2019.1080p.BluRay.x264-GECKOS",
        "1917.2019.2160p.UHD.BluRay.x265-TERMINAL",
        "Tenet.2020.1080p.WEB-DL.DD5.1.H.264-EVO",
        "The.Batman.2022.2160p.WEB-DL.DDP5.1.Atmos.HDR.x265",
        "John.Wick.Chapter.4.2023.2160p.WEB-DL.DDP5.1.Atmos.H265-FLUX",
        "Barbie.2023.1080p.BluRay.x264-PiGNUS",
        "Poor.Things.2023.1080p.WEB-DL.DDP5.1.H264-CMRG",
        "The.Zone.of.Interest.2023.1080p.WEB-DL.DD5.1.H264",
        "Anatomy.of.a.Fall.2023.1080p.BluRay.x264-VOA",
        "Past.Lives.2023.720p.WEB-DL.H264-ETHEL",
        "Killers.of.the.Flower.Moon.2023.2160p.WEB-DL.DDP5.1.HDR.H265",
        "Saltburn.2023.1080p.WEB-DL.DDP5.1.H264-CMRG",
    ];

    for &scene_name in scene_movies {
        let folder = movies_root.join(scene_name);
        fs::create_dir_all(&folder).unwrap();
        touch_file(&folder.join(format!("{}.mkv", scene_name)));
        movie_count += 1;
    }

    // ==========================================
    // 3. CLEAN TV SHOWS (6 shows with seasons)
    // ==========================================

    // Show 1: Breaking Bad (2008) [tvdbid-81189] - 5 seasons (62 episodes)
    {
        let show_dir = tvshows_root.join("Breaking Bad (2008) [tvdbid-81189]");
        fs::create_dir_all(&show_dir).unwrap();
        write_nfo(
            &show_dir.join("tvshow.nfo"),
            &tvshow_nfo("Breaking Bad", 2008, Some(81189), None, Some("tt0903747"), "Drama"),
        );
        touch_file(&show_dir.join("poster.jpg"));
        touch_file(&show_dir.join("backdrop.jpg"));

        let season_counts = [7, 13, 13, 13, 16];
        for (s_idx, &count) in season_counts.iter().enumerate() {
            let season_num = s_idx as u16 + 1;
            let s_dir = show_dir.join(format!("Season {:02}", season_num));
            fs::create_dir_all(&s_dir).unwrap();
            touch_file(&show_dir.join(format!("season{:02}-poster.jpg", season_num)));

            for ep_num in 1..=count {
                // Multi-episode test case for S05
                if season_num == 5 && ep_num == 15 {
                    let ep_stem = "Breaking Bad S05E15-E16 - Granite State & Felina";
                    touch_file(&s_dir.join(format!("{}.mkv", ep_stem)));
                    touch_file(&s_dir.join(format!("{}.en.srt", ep_stem)));
                    episode_count += 2;
                    continue;
                }
                if season_num == 5 && ep_num == 16 {
                    continue; // covered by multi-ep above
                }

                let ep_stem = format!("Breaking Bad S{:02}E{:02} - Episode {:02}", season_num, ep_num, ep_num);
                touch_file(&s_dir.join(format!("{}.mkv", ep_stem)));
                if ep_num <= 3 {
                    touch_file(&s_dir.join(format!("{}.en.srt", ep_stem)));
                    write_nfo(
                        &s_dir.join(format!("{}.nfo", ep_stem)),
                        &episode_nfo("Breaking Bad", season_num, ep_num, &format!("Episode {:02}", ep_num)),
                    );
                }
                episode_count += 1;
            }
        }
        show_count += 1;
    }

    // Show 2: Game of Thrones (2011) [tvdbid-121361] - 2 seasons (20 eps)
    {
        let show_dir = tvshows_root.join("Game of Thrones (2011) [tvdbid-121361]");
        fs::create_dir_all(&show_dir).unwrap();
        write_nfo(
            &show_dir.join("tvshow.nfo"),
            &tvshow_nfo("Game of Thrones", 2011, Some(121361), None, Some("tt0944947"), "Fantasy"),
        );
        touch_file(&show_dir.join("poster.jpg"));

        for season_num in 1..=2 {
            let s_dir = show_dir.join(format!("Season {:02}", season_num));
            fs::create_dir_all(&s_dir).unwrap();
            for ep_num in 1..=10 {
                let ep_stem = format!("Game of Thrones S{:02}E{:02} - Episode {:02}", season_num, ep_num, ep_num);
                touch_file(&s_dir.join(format!("{}.mkv", ep_stem)));
                episode_count += 1;
            }
        }
        show_count += 1;
    }

    // Show 3: Stranger Things (2016) [tvdbid-305288] - 2 seasons (17 eps)
    {
        let show_dir = tvshows_root.join("Stranger Things (2016) [tvdbid-305288]");
        fs::create_dir_all(&show_dir).unwrap();
        write_nfo(
            &show_dir.join("tvshow.nfo"),
            &tvshow_nfo("Stranger Things", 2016, Some(305288), None, Some("tt4574334"), "Sci-Fi"),
        );
        touch_file(&show_dir.join("poster.jpg"));

        for (s_num, count) in [(1, 8), (2, 9)] {
            let s_dir = show_dir.join(format!("Season {:02}", s_num));
            fs::create_dir_all(&s_dir).unwrap();
            for ep_num in 1..=count {
                let ep_stem = format!("Stranger Things S{:02}E{:02} - Chapter {}", s_num, ep_num, ep_num);
                touch_file(&s_dir.join(format!("{}.mkv", ep_stem)));
                episode_count += 1;
            }
        }
        show_count += 1;
    }

    // Show 4: The Mandalorian (2019) [tvdbid-361753] - 2 seasons + Season 00 (17 eps)
    {
        let show_dir = tvshows_root.join("The Mandalorian (2019) [tvdbid-361753]");
        fs::create_dir_all(&show_dir).unwrap();
        write_nfo(
            &show_dir.join("tvshow.nfo"),
            &tvshow_nfo("The Mandalorian", 2019, Some(361753), None, Some("tt8111088"), "Sci-Fi"),
        );
        touch_file(&show_dir.join("poster.jpg"));

        // Season 00 Specials
        let s00_dir = show_dir.join("Season 00");
        fs::create_dir_all(&s00_dir).unwrap();
        touch_file(&s00_dir.join("The Mandalorian S00E01 - Disney Gallery.mkv"));
        episode_count += 1;

        for s_num in 1..=2 {
            let s_dir = show_dir.join(format!("Season {:02}", s_num));
            fs::create_dir_all(&s_dir).unwrap();
            for ep_num in 1..=8 {
                let ep_stem = format!("The Mandalorian S{:02}E{:02} - Chapter {}", s_num, ep_num, (s_num - 1) * 8 + ep_num);
                touch_file(&s_dir.join(format!("{}.mkv", ep_stem)));
                episode_count += 1;
            }
        }
        show_count += 1;
    }

    // Show 5: Arcane (2021) [tvdbid-395256] - 1 season (9 eps)
    {
        let show_dir = tvshows_root.join("Arcane (2021) [tvdbid-395256]");
        fs::create_dir_all(&show_dir).unwrap();
        write_nfo(
            &show_dir.join("tvshow.nfo"),
            &tvshow_nfo("Arcane", 2021, Some(395256), None, Some("tt11126994"), "Animation"),
        );
        let s01_dir = show_dir.join("Season 01");
        fs::create_dir_all(&s01_dir).unwrap();
        for ep in 1..=9 {
            let ep_stem = format!("Arcane S01E{:02} - Act {}", ep, ep);
            touch_file(&s01_dir.join(format!("{}.mkv", ep_stem)));
            episode_count += 1;
        }
        show_count += 1;
    }

    // Show 6: Shōgun (2024) [tmdbid-126308] - 1 season (10 eps)
    {
        let show_dir = tvshows_root.join("Shogun (2024) [tmdbid-126308]");
        fs::create_dir_all(&show_dir).unwrap();
        write_nfo(
            &show_dir.join("tvshow.nfo"),
            &tvshow_nfo("Shogun", 2024, None, Some(126308), None, "Drama"),
        );
        let s01_dir = show_dir.join("Season 01");
        fs::create_dir_all(&s01_dir).unwrap();
        for ep in 1..=10 {
            let ep_stem = format!("Shogun S01E{:02} - Episode {:02}", ep, ep);
            touch_file(&s01_dir.join(format!("{}.mkv", ep_stem)));
            episode_count += 1;
        }
        show_count += 1;
    }

    // ==========================================
    // 4. MESSY SCENE TV SHOWS (4 shows)
    // ==========================================
    let messy_shows: &[(&str, u16)] = &[
        ("the.office.us.s01e{:02}.720p.bluray-lol", 6),
        ("better.call.saul.s01e{:02}.1080p.web-dl.dd5.1.h264", 10),
        ("Chernobyl.S01E{:02}.1080p.AMZN.WEB-DL.DDP5.1.H.264-NTb", 5),
        ("black.mirror.s01e{:02}.720p.bluray.x264", 3),
    ];

    for &(pattern, count) in messy_shows {
        let base_name = pattern.split('.').take(3).collect::<Vec<_>>().join(".");
        let show_dir = tvshows_root.join(&base_name);
        fs::create_dir_all(&show_dir).unwrap();

        for ep in 1..=count {
            let ep_name = pattern.replace("{:02}", &format!("{:02}", ep));
            touch_file(&show_dir.join(format!("{}.mkv", ep_name)));
            episode_count += 1;
        }
        show_count += 1;
    }

    println!("\n✅ Mock Library Generation Complete!");
    println!("   - Movies: {} (Clean + Messy + Multi-Version)", movie_count);
    println!("   - TV Shows: {}", show_count);
    println!("   - Episodes: {}", episode_count);
    println!("   - Output Dir: {}\n", root_dir.display());
}
