use libraw_r::*;
use std::path::Path;

pub fn main() -> anyhow::Result<()> {
    for arg in std::env::args().skip(1) {
        let mut p = Processor::builder()
            .with_params([Params::HalfSize(true)])
            .build();
        p.open(&arg)?;
        println!(
            "Processing {arg} ({:?} {:?})",
            p.idata().make,
            p.idata().model
        );
        p.unpack()?;
        p.dcraw_process()?;
        p.dcraw_ppm_tiff_writer(Path::new(&arg).with_extension(".ppm"))?;
        println!("Writing to {arg}.ppm");
    }
    Ok(())
}
