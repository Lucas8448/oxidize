use std::f64;

pub fn convert(
    ax: f64,
    ay: f64,
    az: f64,
    camera_position: (f64, f64, f64),
    camera_rotation: (f64, f64, f64),
) -> Option<(f64, f64)> {
    // Build rotation matrices (3x3)
    let (cx, cy, cz) = camera_position;
    let (ox, oy, oz) = camera_rotation;

    // m1: rotation around X (note signs preserved from original)
    let m1: [[f64; 3]; 3] = [
        [1.0, 0.0, 0.0],
        [0.0, -ox.cos(), ox.sin()],
        [0.0, -ox.sin(), ox.cos()],
    ];

    // m2: rotation around Y
    let m2: [[f64; 3]; 3] = [
        [oy.cos(), 0.0, -oy.sin()],
        [0.0, 1.0, 0.0],
        [oy.sin(), 0.0, oy.cos()],
    ];

    // m3: rotation around Z
    let m3: [[f64; 3]; 3] = [
        [oz.cos(), oz.sin(), 0.0],
        [-oz.sin(), oz.cos(), 0.0],
        [0.0, 0.0, 1.0],
    ];

    // multiply two 3x3 matrices
    fn mat_mul(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
        let mut r = [[0.0f64; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                let mut sum = 0.0;
                for k in 0..3 {
                    sum += a[i][k] * b[k][j];
                }
                r[i][j] = sum;
            }
        }
        r
    }

    // multiply a row-vector (len 3) on the left by a 3x3 matrix -> row-vector
    fn row_vec_mul(v: [f64; 3], m: &[[f64; 3]; 3]) -> [f64; 3] {
        let mut out = [0.0f64; 3];
        for j in 0..3 {
            let mut sum = 0.0;
            for i in 0..3 {
                sum += v[i] * m[i][j];
            }
            out[j] = sum;
        }
        out
    }

    // combined = m3 * (m2 * m1)
    let temp = mat_mul(&m2, &m1);
    let combined = mat_mul(&m3, &temp);

    // m4 = [ax - cx, ay - cy, az - cz]
    let m4 = [ax - cx, ay - cy, az - cz];

    // d = m4 dot combined
    let d = row_vec_mul(m4, &combined);

    let ex = cx - ax;
    let ey = cy - ay;
    let ez = cz - az;

    // guard against division by zero
    if d[2].abs() < f64::EPSILON {
        return None;
    }

    let x = (ez / d[2]) * d[0] + ex;
    let y = (ez / d[2]) * d[1] + ey;

    Some((x, y))
}