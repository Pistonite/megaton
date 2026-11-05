use megaton_config::config_file::Config;




pub struct Checker {
}

impl Checker {
    #[cu::context("failed to load checker")]
    pub async fn load(config: &Config) -> cu::Result<Checker> {
        // todo
        Ok(Checker {  })
    }
    pub async fn check(self) -> cu::Result<()> {
        // todo
        Ok(())
    }
}

fn is_nso_up_to_date() -> bool {
    false
}
