use crate::version::Version;

pub struct Config {
    app_name: &'static str,
    app_version: Version,
    engine_name: &'static str,
    engine_version: Version,
    enable_validation: bool,
}

impl Config {
    pub fn new(
        app_name: &'static str,
        app_version: Version,
        enable_validation: bool
    ) -> Self {
        let engine_name = "titan";
        Self {
            app_name,
            app_version,
            engine_name,
            engine_version: Version::default(),
            enable_validation
        }
    }

    pub fn app_name(&self) -> &'static str {
        self.app_name
    }

    pub fn app_version(&self) -> &Version {
        &self.app_version
    }

    pub fn engine_name(&self) -> &'static str {
        self.engine_name
    }

    pub fn engine_version(&self) -> &Version {
        &self.engine_version
    }

    pub fn enable_validation(&self) -> bool {
        self.enable_validation
    }
}