use rayon::prelude::*;

/// Concatenates the given slices into a single vector.
pub fn flatten_slices<T: Clone + Send + Sync>(slices: &[&[T]]) -> Vec<T> {
    // Compute offsets and total size of the output so that we can reserve the right
    // amount of memory and then copy the data in parallel.
    let mut offsets = vec![];
    let mut start = 0;
    offsets.push(start);
    for slice in slices {
        start += slice.len();
        offsets.push(start);
    }
    let total_len = start;

    // Allocate the output vector.
    let mut flattened: Vec<T> = Vec::with_capacity(total_len);
    // This is safe as long because all the elements are initialized below.
    unsafe { flattened.set_len(total_len) };
    let mut flattened_slice = flattened.as_mut_slice();

    // Split the output vector into slices for each chunk.
    let mut dst_slices = vec![];
    for slice in slices {
        let (left, right) = flattened_slice.split_at_mut(slice.len());
        dst_slices.push(left);
        flattened_slice = right;
    }

    // Actually copy over the data.
    slices
        .par_iter()
        .zip(dst_slices)
        .for_each(|(src, dst)| unsafe {
            for i in 0..src.len() {
                std::ptr::write(dst.get_unchecked_mut(i), src.get_unchecked(i).clone());
            }
        });

    flattened
}
