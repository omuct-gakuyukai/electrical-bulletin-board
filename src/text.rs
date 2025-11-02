fn count_ascii(s: &str) -> (usize, usize) {
    s.chars().fold((0, 0), |mut acc, c| {
        if c.is_ascii() {
            acc.0 += 1;
        } else {
            acc.1 += 1;
        }
        acc
    })
}

pub fn calc_text_offset(s: &str, tsize: f32, wwidth: f32) -> f32 {
    let (ascii, non_ascii) = count_ascii(s);
    println!("{}", non_ascii * 2 + ascii);
    return ((non_ascii as f32 * tsize) + (ascii as f32 * tsize * 0.48) + wwidth) / 2.0 + 5.0;
}

pub fn calc_speed(w: f32, d: &f32, wwidth: f32) -> f32 {
    return (w + wwidth) / d;
}
