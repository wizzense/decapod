/// Print per-gate timing information.
    #[clap(long, short = 'v')]
    pub verbose: bool,
    /// Automatically refresh specs when staleness is detected
    #[clap(long)]
    pub refresh_specs: bool,
}