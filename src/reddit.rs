use ::configuration::Configuration;
use ::std::fmt::{Display, Formatter, Result as FmtResult};
use rand::{Rng, thread_rng};

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Span {
    Hour,
    Day,
    Week,
    Month,
    Year,
    All,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Mode {
    New,
    Hot,
    Rising,
    Controversial(Span),
    Top(Span),
}

pub fn create_url(config: &Configuration) -> String {
    use reddit::Mode::*;

    let mut subreddits = config.subreddits.clone();
    thread_rng().shuffle(&mut subreddits);

    let subreddit = subreddits.first().unwrap();
    info!("Searching on /r/{}...", subreddit);

    let mut url = format!(
        "https://www.reddit.com/r/{}/{}.json?limit={}",
        subreddit,
        config.mode.identifier(),
        config.query_size
    );

    match config.mode {
        Controversial(span) | Top(span) => url += &format!("&t={}", span),
        _ => (),
    };

    url
}

impl Mode {
    pub fn from_identifier(id: &str, span: Option<&str>) -> Result<Mode, String> {
        let id = id.to_lowercase();
        let id = id.as_ref();

        match id {
            "new" => Ok(Mode::New),
            "hot" => Ok(Mode::Hot),
            "rising" => Ok(Mode::Rising),
            "controversial" => {
                let span_str = span.ok_or_else(|| "-span required")?;
                let span = Span::from_identifier(span_str).ok_or_else(|| "--span invalid")?;
                Ok(Mode::Controversial(span))
            }
            "top" => {
                let span_str = span.ok_or_else(|| "-span required")?;
                let span = Span::from_identifier(span_str).ok_or_else(|| "--span invalid")?;
                Ok(Mode::Top(span))
            }
            unsupported => Err(format!("Unsupported mode '{}'", unsupported)),
        }
    }

    pub fn identifier(&self) -> &'static str {
        use reddit::Mode::*;

        match self {
            New => "new",
            Hot => "hot",
            Rising => "rising",
            Controversial(_) => "controversial",
            Top(_) => "top",
        }
    }
}

impl Span {
    pub fn from_identifier(id: &str) -> Option<Self> {
        use reddit::Span::*;
        match &*id.to_lowercase() {
            "hour" => Some(Hour),
            "day" | "24h" => Some(Day),
            "week" | "7d" => Some(Week),
            "month" => Some(Month),
            "year" | "356d" => Some(Year),
            "all" | "ever" => Some(All),
            _ => None,
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        use reddit::Span::*;

        let to_str = match self {
            Hour => "hour",
            Day => "day",
            Week => "week",
            Month => "month",
            Year => "year",
            All => "all",
        };

        write!(f, "{}", to_str)
    }
}
