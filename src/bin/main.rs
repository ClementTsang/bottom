use bottom::{reset_stdout, start_bottom};

fn main() -> anyhow::Result<()> {
    start_bottom().inspect_err(|_| {
        reset_stdout();
    })
}
