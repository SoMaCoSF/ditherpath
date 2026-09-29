use ditherpath::{run, write_artifacts, DemoConfig, GystFields, GystUuid};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| "demo".into());
    match cmd.as_str() {
        "demo" => demo(args.next().map(PathBuf::from)),
        "uuid" => uuid_cmd(args.collect()),
        "help" | "-h" | "--help" => {
            eprintln!(
                "ditherpath — encoded surface-stable dither + GYST + path\n\n\
                 Commands:\n  demo [outdir]   full loop, write artifacts\n  uuid <key>      print a DITHER_MARK GYST UUIDv8\n"
            );
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("unknown command {other:?}; try help");
            ExitCode::FAILURE
        }
    }
}

fn demo(outdir: Option<PathBuf>) -> ExitCode {
    match run(&DemoConfig::default()) {
        Ok(out) => {
            println!("uuid     {}", out.uuid);
            println!("keepout  {:.2} m", out.payload.keepout_m);
            println!("goal     {:.2}, {:.2}", out.request.goal.x, out.request.goal.y);
            println!("cost     {:.3}", out.route.cost);
            println!("waypoints {}", out.route.polyline.len());
            let dir = outdir.unwrap_or_else(|| PathBuf::from("examples"));
            if let Err(e) = write_artifacts(&out, &dir) {
                eprintln!("artifact write failed: {e}");
                return ExitCode::FAILURE;
            }
            println!(
                "wrote    {}/route.json  route.svg  dither_stack.pgm  uuid.txt",
                dir.display()
            );
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("pipeline failed: {e}");
            ExitCode::FAILURE
        }
    }
}

fn uuid_cmd(args: Vec<String>) -> ExitCode {
    let key = args.first().cloned().unwrap_or_else(|| "pad-00a1".into());
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as u32)
        .unwrap_or(0);
    println!("{}", GystUuid::encode(GystFields::mark(key, ts, 0.5)));
    ExitCode::SUCCESS
}
