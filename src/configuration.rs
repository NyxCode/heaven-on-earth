use reddit::Mode;
use clap::ArgMatches;
use meval::eval_str as str_to_i64;

#[derive(Debug)]
pub struct Configuration {
    pub mode: Mode,
    pub min_ratio: f32,
    pub max_ratio: f32,
    pub query_size: u8,
    pub run_every: Option<String>,
    pub output_dir: String,
    pub random: bool,
}

impl Configuration {
    pub fn from_matches(matches: &ArgMatches) -> Self {
        let mode = matches.value_of("mode").unwrap();
        let span = matches.value_of("span");
        let mode = Mode::from_identifier(mode, span).unwrap();
        let min_ratio = matches
            .value_of("min-ratio")
            .map(|i| str_to_i64(i).unwrap())
            .unwrap() as f32;
        let max_ratio = matches
            .value_of("max-ratio")
            .map(|i| str_to_i64(i).unwrap())
            .unwrap() as f32;
        let query_size = matches
            .value_of("query-size")
            .map(|i| str_to_i64(i).unwrap())
            .unwrap() as u8;
        let run_every = matches.value_of("run-every").map(|expr| expr.to_owned());
        let output_dir = matches.value_of("output-dir").unwrap();
        let random = matches.is_present("random");

        let config = Configuration {
            mode,
            min_ratio,
            max_ratio,
            query_size,
            run_every,
            output_dir: output_dir.to_owned(),
            random,
        };

        info!("{:?}", config);
        config
    }

    pub fn to_command(&self, executable_name: &str) -> String {
        let mut cmd = format!(
            "{} run -m={} --min-ratio={} --max-ratio={} --query-size={} --output-dir={}",
            executable_name,
            self.mode.identifier(),
            self.min_ratio,
            self.max_ratio,
            self.query_size,
            self.output_dir
        );

        match self.mode {
            Mode::Top(span) | Mode::Controversial(span) => {
                cmd.push_str(" --span=");
                cmd.push_str(span.identifier());
            }
            _ => (),
        };

        if let Some(ref cron_expr) = self.run_every {
            cmd.push_str(" --run-every=");
            cmd.push_str(&cron_expr);
        }

        if self.random {
            cmd.push_str(" --random")
        }

        cmd
    }
}