//! Records your own input while you play Minecraft, and fits the human
//! models to it.
//!
//!     rapidbot-recorder record session1.csv     # play; Ctrl+C to stop
//!     rapidbot-recorder analyze session1.csv [sensitivity] > profile.json
//!
//! What is recorded, and only while a window titled "Minecraft..." is in
//! front: raw mouse movement (the same counts the game gets), mouse
//! buttons, and a fixed list of game keys (W A S D, space, shift, ctrl,
//! 1-9, E, Q, F). Other keys are ignored, so chat and passwords are never
//! captured. Each line is `microseconds,kind,a,b`:
//! `m,dx,dy` movement, `b,button,down`, `k,name,down`, `f,focused,0`.
//!
//! Set `RAPIDBOT_SENTRY_DSN` to override the error-reporting target, or to an
//! empty string to disable error reporting.

mod analyze;
#[cfg(windows)]
mod record;

/// Used when `RAPIDBOT_SENTRY_DSN` is not set.
const DEFAULT_SENTRY_DSN: &str =
    "https://023f178131cc96624f4f5b5fd37d2d4b@o4512307547013120.ingest.us.sentry.io/4512307554025472";

/// Returns the DSN to use, or `None` when reporting is disabled.
fn sentry_dsn(override_value: Option<String>) -> Option<String> {
    match override_value {
        Some(dsn) if dsn.is_empty() => None,
        Some(dsn) => Some(dsn),
        None => Some(DEFAULT_SENTRY_DSN.to_owned()),
    }
}

fn main() {
    let dsn = sentry_dsn(std::env::var("RAPIDBOT_SENTRY_DSN").ok());
    let mut sentry_options = sentry::ClientOptions::default();
    sentry_options.release = sentry::release_name!();
    // The recorder only reads movement keys and mouse input. Do not attach
    // request data, user identifiers, or the machine's hostname by default.
    sentry_options.send_default_pii = false;
    sentry_options.server_name = Some("rapidbot-recorder".into());
    let _guard = dsn.map(|dsn| sentry::init((dsn, sentry_options)));

    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        #[cfg(windows)]
        Some("record") => {
            let path = args.get(1).cloned().unwrap_or_else(|| "recording.csv".into());
            if let Err(e) = record::run(&path) {
                eprintln!("recording failed: {e}");
                std::process::exit(1);
            }
        }
        Some("analyze") => {
            let Some(path) = args.get(1) else {
                eprintln!("usage: rapidbot-recorder analyze <file.csv> [sensitivity 0-1, default 0.5]");
                std::process::exit(2);
            };
            let sensitivity = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0.5);
            match analyze::run(path, sensitivity) {
                Ok(report) => println!("{report}"),
                Err(e) => {
                    eprintln!("analysis failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("usage:\n  rapidbot-recorder record <file.csv>\n  rapidbot-recorder analyze <file.csv> [sensitivity]");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentry_dsn_can_be_overridden_or_disabled() {
        assert_eq!(sentry_dsn(None), Some(DEFAULT_SENTRY_DSN.to_owned()));
        assert_eq!(
            sentry_dsn(Some("https://example.invalid/1".to_owned())),
            Some("https://example.invalid/1".to_owned())
        );
        assert_eq!(sentry_dsn(Some(String::new())), None);
    }
}
