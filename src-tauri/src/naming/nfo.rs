use crate::providers::{EpisodeDetails, MovieDetails, TvShowDetails};

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Generates a Jellyfin-compliant `movie.nfo` XML document.
pub fn generate_movie_nfo(movie: &MovieDetails) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?>\n<movie>\n");

    xml.push_str(&format!("  <title>{}</title>\n", escape_xml(&movie.title)));
    if let Some(ref orig) = movie.original_title {
        xml.push_str(&format!("  <originaltitle>{}</originaltitle>\n", escape_xml(orig)));
    }
    xml.push_str(&format!("  <sorttitle>{}</sorttitle>\n", escape_xml(&movie.title)));

    if let Some(year) = movie.year {
        xml.push_str(&format!("  <year>{}</year>\n", year));
    }

    if let Some(ref plot) = movie.overview {
        xml.push_str(&format!("  <plot>{}</plot>\n", escape_xml(plot)));
    }

    if let Some(ref tag) = movie.tagline {
        xml.push_str(&format!("  <tagline>{}</tagline>\n", escape_xml(tag)));
    }

    if let Some(runtime) = movie.runtime {
        xml.push_str(&format!("  <runtime>{}</runtime>\n", runtime));
    }

    if let Some(ref mpaa) = movie.mpaa {
        xml.push_str(&format!("  <mpaa>{}</mpaa>\n", escape_xml(mpaa)));
    }

    // Ratings
    if let Some(rating) = movie.rating {
        let votes = movie.vote_count.unwrap_or(0);
        xml.push_str("  <ratings>\n");
        xml.push_str("    <rating name=\"imdb\" max=\"10\" default=\"true\">\n");
        xml.push_str(&format!("      <value>{:.1}</value>\n", rating));
        xml.push_str(&format!("      <votes>{}</votes>\n", votes));
        xml.push_str("    </rating>\n");
        xml.push_str("  </ratings>\n");
    }

    // Provider IDs (both Kodi/Jellyfin uniqueid and legacy tags)
    if let Some(ref imdb) = movie.provider_ids.imdb_id {
        xml.push_str(&format!("  <uniqueid type=\"imdb\" default=\"true\">{}</uniqueid>\n", imdb));
        xml.push_str(&format!("  <imdbid>{}</imdbid>\n", imdb));
    }
    if let Some(tmdb) = movie.provider_ids.tmdb_id {
        let is_default = if movie.provider_ids.imdb_id.is_none() { " default=\"true\"" } else { "" };
        xml.push_str(&format!("  <uniqueid type=\"tmdb\"{}>{}</uniqueid>\n", is_default, tmdb));
        xml.push_str(&format!("  <tmdbid>{}</tmdbid>\n", tmdb));
    }
    if let Some(tvdb) = movie.provider_ids.tvdb_id {
        xml.push_str(&format!("  <uniqueid type=\"tvdb\">{}</uniqueid>\n", tvdb));
        xml.push_str(&format!("  <tvdbid>{}</tvdbid>\n", tvdb));
    }

    for genre in &movie.genres {
        xml.push_str(&format!("  <genre>{}</genre>\n", escape_xml(genre)));
    }

    if let Some(ref studio) = movie.studio {
        xml.push_str(&format!("  <studio>{}</studio>\n", escape_xml(studio)));
    }

    if let Some(ref director) = movie.director {
        xml.push_str(&format!("  <director>{}</director>\n", escape_xml(director)));
    }

    for writer in &movie.writers {
        xml.push_str(&format!("  <credits>{}</credits>\n", escape_xml(writer)));
    }

    for actor in &movie.actors {
        xml.push_str("  <actor>\n");
        xml.push_str(&format!("    <name>{}</name>\n", escape_xml(&actor.name)));
        if let Some(ref role) = actor.role {
            xml.push_str(&format!("    <role>{}</role>\n", escape_xml(role)));
        }
        xml.push_str(&format!("    <order>{}</order>\n", actor.order));
        if let Some(ref thumb) = actor.thumb {
            xml.push_str(&format!("    <thumb>{}</thumb>\n", escape_xml(thumb)));
        }
        xml.push_str("  </actor>\n");
    }

    xml.push_str("</movie>\n");
    xml
}

/// Generates a Jellyfin-compliant `tvshow.nfo` XML document.
pub fn generate_tvshow_nfo(show: &TvShowDetails, display_order: &str) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?>\n<tvshow>\n");

    xml.push_str(&format!("  <title>{}</title>\n", escape_xml(&show.title)));
    if let Some(ref orig) = show.original_title {
        xml.push_str(&format!("  <originaltitle>{}</originaltitle>\n", escape_xml(orig)));
    }
    xml.push_str(&format!("  <sorttitle>{}</sorttitle>\n", escape_xml(&show.title)));

    if let Some(year) = show.year {
        xml.push_str(&format!("  <year>{}</year>\n", year));
    }

    if let Some(ref status) = show.status {
        xml.push_str(&format!("  <status>{}</status>\n", escape_xml(status)));
    }

    if let Some(ref plot) = show.overview {
        xml.push_str(&format!("  <plot>{}</plot>\n", escape_xml(plot)));
    }

    if let Some(ref mpaa) = show.mpaa {
        xml.push_str(&format!("  <mpaa>{}</mpaa>\n", escape_xml(mpaa)));
    }

    xml.push_str(&format!("  <displayorder>{}</displayorder>\n", display_order));

    if let Some(rating) = show.rating {
        let votes = show.vote_count.unwrap_or(0);
        xml.push_str("  <ratings>\n");
        xml.push_str("    <rating name=\"imdb\" max=\"10\" default=\"true\">\n");
        xml.push_str(&format!("      <value>{:.1}</value>\n", rating));
        xml.push_str(&format!("      <votes>{}</votes>\n", votes));
        xml.push_str("    </rating>\n");
        xml.push_str("  </ratings>\n");
    }

    if let Some(tvdb) = show.provider_ids.tvdb_id {
        xml.push_str(&format!("  <uniqueid type=\"tvdb\" default=\"true\">{}</uniqueid>\n", tvdb));
        xml.push_str(&format!("  <tvdbid>{}</tvdbid>\n", tvdb));
    }
    if let Some(ref imdb) = show.provider_ids.imdb_id {
        let is_default = if show.provider_ids.tvdb_id.is_none() { " default=\"true\"" } else { "" };
        xml.push_str(&format!("  <uniqueid type=\"imdb\"{}>{}</uniqueid>\n", is_default, imdb));
        xml.push_str(&format!("  <imdbid>{}</imdbid>\n", imdb));
    }
    if let Some(tmdb) = show.provider_ids.tmdb_id {
        xml.push_str(&format!("  <uniqueid type=\"tmdb\">{}</uniqueid>\n", tmdb));
        xml.push_str(&format!("  <tmdbid>{}</tmdbid>\n", tmdb));
    }

    for genre in &show.genres {
        xml.push_str(&format!("  <genre>{}</genre>\n", escape_xml(genre)));
    }

    if let Some(ref studio) = show.studio {
        xml.push_str(&format!("  <studio>{}</studio>\n", escape_xml(studio)));
    }

    for actor in &show.actors {
        xml.push_str("  <actor>\n");
        xml.push_str(&format!("    <name>{}</name>\n", escape_xml(&actor.name)));
        if let Some(ref role) = actor.role {
            xml.push_str(&format!("    <role>{}</role>\n", escape_xml(role)));
        }
        xml.push_str(&format!("    <order>{}</order>\n", actor.order));
        if let Some(ref thumb) = actor.thumb {
            xml.push_str(&format!("    <thumb>{}</thumb>\n", escape_xml(thumb)));
        }
        xml.push_str("  </actor>\n");
    }

    xml.push_str("</tvshow>\n");
    xml
}

/// Generates a Jellyfin-compliant `<EpisodeFileName>.nfo` XML document.
pub fn generate_episode_nfo(
    episode: &EpisodeDetails,
    show_title: &str,
    airs_before_season: Option<u32>,
    airs_before_episode: Option<u32>,
) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?>\n<episodedetails>\n");

    let ep_title = episode.title.as_deref().unwrap_or("Episode");
    xml.push_str(&format!("  <title>{}</title>\n", escape_xml(ep_title)));
    xml.push_str(&format!("  <showtitle>{}</showtitle>\n", escape_xml(show_title)));
    xml.push_str(&format!("  <season>{}</season>\n", episode.season));
    xml.push_str(&format!("  <episode>{}</episode>\n", episode.episode));

    if let Some(ref aired) = episode.aired_date {
        xml.push_str(&format!("  <aired>{}</aired>\n", escape_xml(aired)));
    }

    if let Some(ref plot) = episode.overview {
        xml.push_str(&format!("  <plot>{}</plot>\n", escape_xml(plot)));
    }

    if let Some(rating) = episode.rating {
        let votes = episode.vote_count.unwrap_or(0);
        xml.push_str("  <ratings>\n");
        xml.push_str("    <rating name=\"imdb\" max=\"10\" default=\"true\">\n");
        xml.push_str(&format!("      <value>{:.1}</value>\n", rating));
        xml.push_str(&format!("      <votes>{}</votes>\n", votes));
        xml.push_str("    </rating>\n");
        xml.push_str("  </ratings>\n");
    }

    if let Some(ref imdb) = episode.provider_ids.imdb_id {
        xml.push_str(&format!("  <uniqueid type=\"imdb\" default=\"true\">{}</uniqueid>\n", imdb));
    }
    if let Some(tvdb) = episode.provider_ids.tvdb_id {
        xml.push_str(&format!("  <uniqueid type=\"tvdb\">{}</uniqueid>\n", tvdb));
    }
    if let Some(tmdb) = episode.provider_ids.tmdb_id {
        xml.push_str(&format!("  <uniqueid type=\"tmdb\">{}</uniqueid>\n", tmdb));
    }

    if let Some(ref dir) = episode.director {
        xml.push_str(&format!("  <director>{}</director>\n", escape_xml(dir)));
    }

    if let Some(ref writer) = episode.writer {
        xml.push_str(&format!("  <credits>{}</credits>\n", escape_xml(writer)));
    }

    // Special episode ordering tags for Season 00
    if episode.season == 0 {
        if let Some(s) = airs_before_season {
            xml.push_str(&format!("  <airsbefore_season>{}</airsbefore_season>\n", s));
        }
        if let Some(e) = airs_before_episode {
            xml.push_str(&format!("  <airsbefore_episode>{}</airsbefore_episode>\n", e));
        }
    }

    xml.push_str("</episodedetails>\n");
    xml
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::ProviderIdMap;

    #[test]
    fn test_movie_nfo_structure() {
        let movie = MovieDetails {
            provider_ids: ProviderIdMap {
                imdb_id: Some("tt0078748".into()),
                tmdb_id: Some(348),
                tvdb_id: None,
            },
            title: "Alien".into(),
            original_title: Some("Alien".into()),
            tagline: Some("In space no one can hear you scream".into()),
            overview: Some("A commercial crew aboard the Nostromo...".into()),
            year: Some(1979),
            runtime: Some(117),
            genres: vec!["Sci-Fi".into(), "Horror".into()],
            mpaa: Some("R".into()),
            studio: Some("20th Century Fox".into()),
            director: Some("Ridley Scott".into()),
            writers: vec!["Dan O'Bannon".into()],
            actors: Vec::new(),
            poster_url: None,
            backdrop_url: None,
            rating: Some(8.5),
            vote_count: Some(900000),
        };

        let nfo = generate_movie_nfo(&movie);
        assert!(nfo.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\" standalone=\"yes\"?>"));
        assert!(nfo.contains("<uniqueid type=\"imdb\" default=\"true\">tt0078748</uniqueid>"));
        assert!(nfo.contains("<uniqueid type=\"tmdb\">348</uniqueid>"));
        assert!(nfo.contains("<imdbid>tt0078748</imdbid>"));
        assert!(nfo.contains("<tmdbid>348</tmdbid>"));
        assert!(nfo.contains("<title>Alien</title>"));
    }
}
