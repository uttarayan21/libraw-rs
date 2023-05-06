use libraw_r::thumbnail::Thumb;
use libraw_r::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for arg in std::env::args().skip(1) {
        let p = EmptyProcessor::default();
        let p: Processor = p.open(&arg)?;
        let mut p = p.thumbnail_processor();

        dbg!(p.list());
        dbg!(p.count());
        dbg!(p.unpack(None)?);
        let t = dbg!(p.get()?);
        std::fs::write("file.jpg", t.data)?;
    }
    Ok(())
}
