//! `sim-cli`: the headless lab (PLAN §13.3).

mod arch_lab;
mod cage_lab;
mod pa_lab;
mod pitch_lab;
mod sens_lab;
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
    /// Contact map: placement deviation → exit velocity, launch angle, spray, spin (M2).
    CageMap(cage_lab::MapArgs),
    /// Noisy pitches against noisy swings with perfect perception (M2).
    Cage(cage_lab::CageArgs),
    /// Plate appearances with perception and decisions: B1 rates (M3).
    Pa(pa_lab::PaArgs),
    /// One-attribute sweeps with common random numbers; checks directions (M3).
    Sensitivity(sens_lab::SensArgs),
    /// B1 profiles of batter and pitcher archetypes (M3).
    Archetypes(arch_lab::ArchArgs),
}

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Pitch(a) => pitch_lab::pitch(a),
        Cmd::Tunnel(a) => pitch_lab::tunnel(a),
        Cmd::Arsenal(a) => pitch_lab::arsenal(a),
        Cmd::CageMap(a) => cage_lab::map(a),
        Cmd::Cage(a) => cage_lab::cage(a),
        Cmd::Pa(a) => pa_lab::pa(a),
        Cmd::Sensitivity(a) => sens_lab::sensitivity(a),
        Cmd::Archetypes(a) => arch_lab::run(a),
    }
}
