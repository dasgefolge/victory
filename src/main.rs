//#![deny(rust_2018_idioms, unused, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]

use {
    std::{
        fmt,
        io::{
            self,
            prelude::*,
            stdin,
            stdout,
        },
    },
    structopt::StructOpt,
    victory::state::{
        Input,
        InputRequest,
        MetaInput,
        State,
    },
};

#[derive(StructOpt)]
struct Args {
    #[structopt(long = "debug")]
    debug: bool,
}

fn input(prompt: impl fmt::Display) -> io::Result<String> {
    print!("{}: ", prompt);
    stdout().flush()?;
    let mut buf = String::default();
    stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_owned())
}

#[paw::main]
fn main(args: Args) -> io::Result<()> {
    let mut state = State::default();
    loop {
        if args.debug { eprintln!("{:#?}", state) }
        state.advance_game(match state.next_input() {
            InputRequest::Meta => Input::Meta({
                let name = input("enter player name to add/remove [leave blank to start]")?;
                if name.is_empty() {
                    MetaInput::Go
                } else if state.players().iter().any(|player| player.id == name) {
                    MetaInput::Quit(name)
                } else {
                    MetaInput::Join(name, None)
                }
            }),
            _ => unimplemented!(), //TODO
        })
    }
}
