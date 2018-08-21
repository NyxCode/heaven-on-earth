#[allow(dead_code)]
#[derive(Debug)]
pub enum Span {
    Hour,
    Day,
    Week,
    Month,
    Year,
    All,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Mode {
    New,
    Hot,
    Top(Span),
}

impl Mode {
    pub fn to_url(&self) -> String {
        use reddit::Mode::*;

        let mut url = "https://www.reddit.com/r/EarthPorn/".to_owned();
        match self {
            New => url.push_str("new.json"),
            Hot => url.push_str("hot.json"),
            Top(span) => {
                url.push_str("top.json?t=");
                url.push_str(span.identifier());
            }
        };
        url
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
            _ => None
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
            All => "all"
        }
    }
}