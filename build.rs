use anyhow::Result;

fn main() -> Result<()> {
    slint_build::compile("src/ui/appwindow.slint")?;
    Ok(())
}
