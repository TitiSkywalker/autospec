//! ReLU (Rectified Linear Unit) activation function.
//!
//! For each element x, ReLU is defined as: `relu(x) = max(0, x)`.

/// Apply ReLU element-wise to `array`, consuming and returning the result.
///
/// For each element: `y[i] = if x[i] > 0.0 { x[i] } else { 0.0 }`.
pub fn relu(mut array: Vec<f64>) -> Vec<f64> {
    let len = array.len();
    let mut i: usize = 0;
    while i < len {
        if array[i] <= 0.0 {
            array[i] = 0.0;
        }
        i += 1;
    }
    array
}