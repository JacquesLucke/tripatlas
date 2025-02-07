use rayon::prelude::*;

pub fn flatten_slices<T: Clone + Send + Sync>(slices: &[&[T]]) -> Vec<T> {
    let mut offsets = vec![];
    let mut start = 0;
    offsets.push(start);
    for slice in slices {
        start += slice.len();
        offsets.push(start);
    }
    let total_len = start;

    let mut flattened: Vec<T> = Vec::with_capacity(total_len);
    unsafe { flattened.set_len(total_len) };
    let mut flattened_slice = flattened.as_mut_slice();
    let mut dst_slices = vec![];
    for slice in slices {
        let (left, right) = flattened_slice.split_at_mut(slice.len());
        dst_slices.push(left);
        flattened_slice = right;
    }

    slices.par_iter().zip(dst_slices).for_each(|(src, dst)| {
        dst.clone_from_slice(src);
    });

    flattened
}
