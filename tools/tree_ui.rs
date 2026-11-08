/// layout of the skill tree screen: one school per tab, three paths side by side.
/// Returns the (x, y) of the node's top-left corner and the box size.
pub fn node_pos(i: usize, fw: i32, _fh: i32) -> (i32, i32, i32) {
    let n = &NODES[i];
    let bs = 20;
    let x = fw / 2 + (n.lane as i32 - 1) * 150 - 60;
    let y = 50 + n.row as i32 * 31;
    (x, y, bs)
}
/// the tabs along the top
fn tab_rect(k: usize, fw: i32) -> (i32, i32, i32, i32) {
    let tw = (fw - 20) / 6;
    (10 + k as i32 * tw, 28, tw - 2, 14)
}
pub fn tab_at(m: (f32, f32), view: (f32, f32)) -> Option<usize> {
    let fw = view.0 as i32;
    (0..6).find(|&k| {
        let (x, y, w, h) = tab_rect(k, fw);
        let (mx, my) = (m.0 as i32, m.1 as i32);
        mx >= x && mx < x + w && my >= y && my < y + h
    })
}
pub fn node_at(m: (f32, f32), view: (f32, f32), tab: usize) -> Option<usize> {
    let (fw, fh) = (view.0 as i32, view.1 as i32);
    for i in 0..N {
        if NODES[i].col != tab {
            continue;
        }
        let (x, y, bs) = node_pos(i, fw, fh);
        // the name to the right of the box counts too
        if m.0 as i32 >= x - 2 && m.0 as i32 <= x + bs + 70 && m.1 as i32 >= y - 2 && m.1 as i32 <= y + bs + 2 {
            return Some(i);
        }
    }
    None
}
