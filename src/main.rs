use signal_hook::{consts::{SIGUSR1, SIGUSR2, SIGINT, SIGTERM}, iterator::Signals};
use std::{error::Error};
use std::process;
use std::process::Command;
use std::os::unix::fs::PermissionsExt;
use std::fs::File;
use std::fs;
use std::io::Write;

#[derive(Debug)]
enum ChargerState {
  Plugged,
  Unplugged
}

#[derive(Debug)]
enum LidState {
    Opened,
    Closed
}

fn read_charger_state() -> Result<(ChargerState), Box<dyn Error>> {
  let contents = fs::read_to_string("/sys/class/power_supply/AC0/online")?;
  match contents.trim().parse::<i32>().unwrap() {
      1 => Ok(ChargerState::Plugged),
      0 => Ok(ChargerState::Unplugged),
      _ => unreachable!()
  }
}

fn handle_lid_event(event: (LidState, ChargerState)) -> Result<(), Box<dyn Error>> {
    match event {
        (LidState::Opened, ChargerState::Plugged) => {
            Command::new("autorandr")
                .args(["--load", "docked"])
                .output()
                .expect("failed to execute process");
        },
        (LidState::Opened, ChargerState::Unplugged) => {
            Command::new("autorandr")
                .args(["--load", "mobile"])
                .output()
                .expect("failed to execute process");
        },
        (LidState::Closed, ChargerState::Plugged) => {
            Command::new("autorandr")
                .args(["--load", "lid_closed"])
                .output()
                .expect("failed to execute process");

        },
        (LidState::Closed, ChargerState::Unplugged) => {
            Command::new("autorandr")
                .args(["--load", "mobile"])
                .output()
                .expect("failed to execute process");
        },
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut signals = Signals::new(&[SIGUSR1, SIGUSR2, SIGINT, SIGTERM])?;

    let filename = "/tmp/lid_handler";
    let mut file = File::create(filename)?;
    let mut perms = file.metadata()?.permissions();
    perms.set_mode(0o644);
    file.set_permissions(perms)?;
    file.write_all(process::id().to_string().as_bytes())?;

    for sig in signals.forever() {
        match sig {
            SIGUSR1 => {
                handle_lid_event((LidState::Opened, read_charger_state()?))?;
            },
            SIGUSR2 => { 
                handle_lid_event((LidState::Closed, read_charger_state()?))?;
            },
            SIGINT | SIGTERM => {
                println!("terminating...");
                fs::remove_file(filename)?;
                break
            },
            _ => unreachable!(),
        }
    }


    Ok(())
}
