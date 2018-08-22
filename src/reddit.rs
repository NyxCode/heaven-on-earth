#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum Span {
    Hour,
    Day,
    Week,
    Month,
    Year,
    All,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum Mode {
    New,
    Hot,
    Rising,
    Controversial(Span),
    Top(Span),
}

impl Mode {
    pub fn to_url(&self) -> String {
        use reddit::Mode::*;

        let mut url = "https://www.reddit.com/r/EarthPorn/".to_owned();
        match self {
            New => url.push_str("new.json"),
            Hot => url.push_str("hot.json"),
            Rising => url.push_str("rising.json"),
            Controversial(span) => {
                url.push_str("controversial.json?t=");
                url.push_str(span.identifier());
            }
            Top(span) => {
                url.push_str("top.json?t=");
                url.push_str(span.identifier());
            }
        };
        url
    }

    pub fn from_identifier(id: &str, span: Option<&str>) -> Option<Mode> {
        let id = id.to_lowercase();
        let id = id.as_ref();
        match id {
            "new" => Some(Mode::New),
            "hot" => Some(Mode::Hot),
            "rising" => Some(Mode::Rising),
            "controversial" => span
                .and_then(Span::from_identifier)
                .map(Mode::Controversial),
            "top" => span.and_then(Span::from_identifier).map(Mode::Top),
            unsupported => {
                error!("Unsupported mode '{}'", unsupported);
                None
            }
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

    pub fn identifier(&self) -> &'static str {
        use reddit::Span::*;
        match self {
            Hour => "hour",
            Day => "day",
            Week => "week",
            Month => "month",
            Year => "year",
            All => "all",
        }
    }
}
