use crate::error::WpsError;

const INTRO: &str = include_str!("../art/intro.txt");
const FEATURES: &str = include_str!("../art/features.txt");
const SETUP: &str = include_str!("../art/setup.txt");
const CONFIG: &str = include_str!("../art/config.txt");
const FORMAT: &str = include_str!("../art/format.txt");
const OUTRO: &str = include_str!("../art/outro.txt");
const FRAMEWORK: &str = include_str!("../art/framework.txt");

pub fn show(scene: &str) -> Result<(), WpsError> {
    let content = match scene {
        "intro" => INTRO,
        "features" => FEATURES,
        "setup" => SETUP,
        "config" => CONFIG,
        "format" => FORMAT,
        "outro" => OUTRO,
        "framework" => FRAMEWORK,
        "all" | "guide" => {
            print!("\x1B[2J\x1B[1;1H");
            println!("{INTRO}");
            println!("{FEATURES}");
            println!("{FRAMEWORK}");
            println!("{SETUP}");
            println!("{CONFIG}");
            println!("{FORMAT}");
            println!("{OUTRO}");
            return Ok(());
        }
        other => {
            return Err(WpsError::Validation(format!(
                "unknown scene `{other}`; use one of: intro, features, framework, setup, config, format, outro, all"
            )));
        }
    };
    print!("\x1B[2J\x1B[1;1H");
    println!("{content}");
    Ok(())
}
