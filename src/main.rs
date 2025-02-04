use std::{
    env,
    f32::{MAX, MIN},
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use ndarray::Array2;
use ndarray_npy::read_npy;

fn mesh_from_file<P: AsRef<Path>>(path: P) -> Array2<f32> {
    let grid = read_npy(path).unwrap();
    grid
}

fn main() {
    let grid: Array2<f32> = mesh_from_file(format!(
        "{}{}",
        &env::var("CARGO_MANIFEST_DIR").unwrap(),
        "/256_scalar_field.npy"
    ));

    pollster::block_on(navier_map::run(
        grid.as_slice().unwrap(),
        grid.shape()[0] as u32,
        grid.shape()[1] as u32,
    ));
}
