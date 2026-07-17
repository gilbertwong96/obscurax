mod atoms;
mod browser;
mod callback;
mod cookie;
mod error;
mod page;
mod page_thread;

#[rustler::nif]
fn hello() -> rustler::Atom {
    atoms::world()
}

rustler::init!("Elixir.Obscurax.Nif");
