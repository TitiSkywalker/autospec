/// Failure: empty matrix panics — bidiagonal cannot decompose a 0×0 matrix.
#[test]
#[should_panic(expected = "assertion")]
fn bidiagonal_empty_panics() {
    let mut matrix: Vec<f64> = vec![];
    let nrows = 0;
    let ncols = 0;
    let min_nm = 0;
    let mut diagonal = vec![0.0; min_nm];
    let mut off_diagonal = vec![0.0; if min_nm > 0 { min_nm - 1 } else { 0 }];
    let mut col_axis_work = vec![0.0; nrows];
    let mut row_axis_work = vec![0.0; ncols];
    let mut work = vec![0.0; nrows];
    let mut sub_work = vec![0.0; nrows * ncols];
    let _ = matrix_bidiagonal::bidiagonal_decompose(
        &mut matrix, nrows, ncols,
        &mut diagonal, &mut off_diagonal,
        &mut col_axis_work, &mut row_axis_work, &mut work, &mut sub_work,
    );
}
