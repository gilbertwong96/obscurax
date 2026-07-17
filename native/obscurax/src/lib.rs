mod atoms;
mod error;

#[rustler::nif]
fn hello() -> rustler::Atom {
    atoms::world()
}

rustler::init!("Elixir.Obscurax.Nif");
