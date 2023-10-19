pub use revm::primitives::kzg::{format_kzg_settings, G1Points, G2Points, KzgErrors};
use std::path::PathBuf;
use std::{env, fs};
use structopt::StructOpt;

/// Statetest command
#[derive(StructOpt, Debug)]
pub struct Cmd {
    /// Input path to the kzg trusted setup file.
    #[structopt(required = true)]
    path: PathBuf,
    /// path to output g1 point in binary format.
    #[structopt(long)]
    g1: Option<PathBuf>,
    /// Path to output g2 point in binary format.
    #[structopt(long)]
    g2: Option<PathBuf>,
}

impl Cmd {
    /// Run statetest command.
    pub fn run(&self) -> Result<(), KzgErrors> {
        // check if path exists.
        if !self.path.exists() {
            return Err(KzgErrors::PathNotExists);
        }

        let out_dir = env::current_dir().map_err(|_| KzgErrors::FailedCurrentDirectory)?;

        let kzg_trusted_settings =
            fs::read_to_string(&self.path).map_err(|_| KzgErrors::NotValidFile)?;

        // format points
        let (g1, g2) = format_kzg_settings(&kzg_trusted_settings)?;

        let g1_path = self
            .g1
            .clone()
            .unwrap_or_else(|| out_dir.join("g1_points.bin"));

        let g2_path = self
            .g2
            .clone()
            .unwrap_or_else(|| out_dir.join("g2_points.bin"));

        // output points
        fs::write(&g1_path, into_flattened(g1.to_vec())).map_err(|_| KzgErrors::IOError)?;
        fs::write(&g2_path, into_flattened(g2.to_vec())).map_err(|_| KzgErrors::IOError)?;
        println!("Finished formatting kzg trusted setup into binary representation.");
        println!("G1 point path: {:?}", g1_path);
        println!("G2 point path: {:?}", g2_path);
        Ok(())
    }
}

/// [`Vec::into_flattened`].
fn into_flattened<T, const N: usize>(vec: Vec<[T; N]>) -> Vec<T> {
    let (ptr, len, cap) = into_raw_parts(vec);
    let (new_len, new_cap) = if core::mem::size_of::<T>() == 0 {
        (len.checked_mul(N).expect("vec len overflow"), usize::MAX)
    } else {
        // SAFETY:
        // - `cap * N` cannot overflow because the allocation is already in
        // the address space.
        // - Each `[T; N]` has `N` valid elements, so there are `len * N`
        // valid elements in the allocation.
        unsafe {
            (
                len.checked_mul(N).unwrap_unchecked(),
                cap.checked_mul(N).unwrap_unchecked(),
            )
        }
    };
    // SAFETY:
    // - `ptr` was allocated by `self`
    // - `ptr` is well-aligned because `[T; N]` has the same alignment as `T`.
    // - `new_cap` refers to the same sized allocation as `cap` because
    // `new_cap * size_of::<T>()` == `cap * size_of::<[T; N]>()`
    // - `len` <= `cap`, so `len * N` <= `cap * N`.
    unsafe { Vec::from_raw_parts(ptr.cast(), new_len, new_cap) }
}

/// [`Vec::into_raw_parts`]
fn into_raw_parts<T>(vec: Vec<T>) -> (*mut T, usize, usize) {
    let mut me = core::mem::ManuallyDrop::new(vec);
    (me.as_mut_ptr(), me.len(), me.capacity())
}
