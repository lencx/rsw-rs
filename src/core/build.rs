//! rsw build

use std::path::PathBuf;
use std::process::Command;

use crate::config::CrateConfig;
use crate::core::Link;
use crate::core::RswInfo;
use crate::utils::{get_crate_metadata, get_pkg, print, rsw_watch_file};

pub struct Build {
    config: CrateConfig,
    rsw_type: String,
    cli: String,
}

impl Build {
    pub fn new(config: CrateConfig, rsw_type: &str, cli: String) -> Build {
        Build {
            config,
            rsw_type: rsw_type.into(),
            cli,
        }
    }

    pub fn init(&self) -> bool {
        let config = &self.config;
        let rsw_type = &self.rsw_type;
        let name = &config.name;
        let out_dir = config.out_dir.as_ref().unwrap();
        let target = config.target.as_ref().unwrap();
        let mut args = vec!["build", name, "--out-dir", out_dir, "--target", target];

        // profile
        let mut profile = config.build.as_ref().unwrap().profile.as_ref().unwrap();
        if rsw_type == "watch" {
            profile = config.watch.as_ref().unwrap().profile.as_ref().unwrap();
        }
        let arg_profile = format!("--{}", profile);
        args.push(&arg_profile);

        // scope
        let (_, scope) = get_pkg(&self.config.name);
        if scope != "" {
            args.push("--scope");
            args.push(scope.as_str());
        }

        let metadata = get_crate_metadata(name.as_str());
        info!("🚧  wasm-pack {}", args.join(" "));

        let status = Command::new("wasm-pack")
            .args(&args)
            .status()
            .expect("failed to execute process");

        println!("");

        let mut is_ok = true;

        match status.success() {
            true => print(RswInfo::CrateOk(
                name.into(),
                rsw_type.into(),
                metadata["package"]["version"].to_string(),
            )),
            false => {
                let output = Command::new("wasm-pack")
                    .args(&args)
                    .stderr(std::process::Stdio::piped())
                    .output()
                    .unwrap();
                let err = std::str::from_utf8(&output.stderr).unwrap();
                let content = [
                    "[RSW::ERROR]: ",
                    name,
                    "\n\n[RSW::ARGS]: wasm-pack ",
                    &args.join(" "),
                    "\n\n[RSW::BUILD]:\n",
                    err,
                ];
                rsw_watch_file(content.concat().as_bytes()).unwrap();
                print(RswInfo::CrateFail(name.into(), rsw_type.into()));

                is_ok = false;
            }
        }

        // TODO: link
        if config.link.unwrap() {
            let cli = &self.cli;
            Link::new(
                cli.into(),
                PathBuf::from(name).join(out_dir),
                name.to_string(),
            )
            .init();
        }

        print(RswInfo::SplitLine);

        is_ok
    }
}
