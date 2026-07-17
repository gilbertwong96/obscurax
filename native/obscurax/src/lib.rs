mod atoms {
    rustler::atoms! {
        world,
    }
}

#[rustler::nif]
fn hello() -> rustler::Atom {
    atoms::world()
}

rustler::init!("Elixir.Obscurax.Nif");
