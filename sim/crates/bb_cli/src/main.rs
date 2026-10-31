//! `sim-cli`: the headless lab (PLAN §13.3).

mod pitch_lab;
#[cfg(feature = "rerun")]
mod rr;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sim-cli", version, about = "3D Baseball simulation lab")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Throw pitches with execution noise and print Statcast-style telemetry.
    Pitch(pitch_lab::PitchArgs),
    /// Tunneling: pair a fastball with a second pitch that matches it at the tunnel point.
    Tunnel(pitch_lab::TunnelArgs),
    /// Print the arsenal templates' nominal shapes and movement for a pitcher.
    Arsenal(pitch_lab::ArsenalArgs),
}

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Pitch(a) => pitch_lab::pitch(a),
        Cmd::Tunnel(a) => pitch_lab::tunnel(a),
        Cmd::Arsenal(a) => pitch_lab::arsenal(a),
    }
}
