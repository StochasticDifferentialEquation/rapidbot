//! Creative flight check: double-taps jump to start flying, climbs, hovers,
//! sprints forward, then descends until it lands (which ends flight). For
//! checking `LocalPlayer.aiStep` / `Player.travel` flight against a local
//! server. The bot needs `may_fly`, so put it in creative first:
//!
//!     gamemode creative Walker
//!     cargo run --release -p rapidbot-client --example fly -- localhost:25565 Walker
//!
//! Watch the log: `flying` should turn true after the double tap, `y` should
//! rise, hold steady while hovering, and the server should not pull the bot
//! back or kick it ("Flying is not enabled on this server").

use rapidbot_client::Keys;
use rapidbot_client::{Account, ClientConfig, Controller, TickContext};

/// Ticks after joining before the script starts (chunks need to load).
const START: u64 = 100;

struct Fly;

impl Controller for Fly {
    fn tick(&mut self, ctx: &mut TickContext<'_>) {
        let p = &*ctx.player;
        if ctx.tick % 10 == 0 {
            tracing::info!(
                tick = ctx.tick,
                y = p.pos.y,
                vy = p.delta_movement.y,
                flying = p.abilities.flying,
                may_fly = p.abilities.may_fly,
                on_ground = p.on_ground,
                "fly"
            );
        }
        let t = ctx.tick;
        let keys = match t {
            // Double tap: press, release for three ticks, press again.
            t if t == START || t == START + 4 => Keys {
                jump: true,
                ..Keys::default()
            },
            t if t < START + 5 => Keys::default(),
            // Climb.
            t if t < START + 45 => Keys {
                jump: true,
                ..Keys::default()
            },
            // Hover: no input, y must stay put.
            t if t < START + 105 => Keys::default(),
            // Sprint forward (flying sprint doubles the flying speed).
            t if t < START + 145 => Keys {
                forward: true,
                sprint: true,
                ..Keys::default()
            },
            // Descend until the landing ends flight.
            _ if p.abilities.flying => Keys {
                shift: true,
                ..Keys::default()
            },
            _ => {
                if t % 20 == 0 {
                    tracing::info!(on_ground = p.on_ground, "landed, flight ended");
                }
                Keys::default()
            }
        };
        ctx.set_keys(keys);
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let mut args = std::env::args().skip(1);
    let address = args.next().unwrap_or_else(|| "localhost".into());
    let name = args.next().unwrap_or_else(|| "Walker".into());
    let reason =
        rapidbot_client::run_with(ClientConfig::new(address, Account::Offline { name }), Fly).await;
    tracing::warn!("connection ended: {reason}");
}
