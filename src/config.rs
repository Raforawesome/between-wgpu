//! Small dependency-free CLI for reproducible demonstrations.
#[derive(Debug)]
pub struct Config {
    pub seed: i32,
    pub view: i32,
    pub height: i32,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            seed: 42,
            view: crate::constants::VIEW_DISTANCE,
            height: crate::constants::VIEW_DISTANCE_Y,
        }
    }
}
impl Config {
    pub fn parse(args: impl IntoIterator<Item = String>) -> anyhow::Result<Option<Self>> {
        let mut config = Self::default();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            if arg == "--help" || arg == "-h" {
                return Ok(None);
            }
            anyhow::ensure!(
                matches!(
                    arg.as_str(),
                    "--seed" | "--view-distance" | "--vertical-distance"
                ),
                "unknown argument: {arg}"
            );
            let value = args
                .next()
                .ok_or_else(|| anyhow::anyhow!("missing value for {arg}"))?
                .parse::<i32>()?;
            match arg.as_str() {
                "--seed" => config.seed = value,
                "--view-distance" => config.view = value,
                _ => config.height = value,
            }
        }
        anyhow::ensure!(
            (1..=128).contains(&config.view),
            "view distance must be 1..=128 chunks"
        );
        anyhow::ensure!(
            (1..=32).contains(&config.height),
            "vertical distance must be 1..=32 chunks"
        );
        Ok(Some(config))
    }
}
