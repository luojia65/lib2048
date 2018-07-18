use rand::prelude::*;
use Board;
use TilePos;

// 90% is 1u8(shown as 2), 10% is 2u8(shown as 4).
pub(crate) fn tile_value() -> u8 {
    if thread_rng().gen_bool(0.9) { 1 } else { 2 }
}

pub(crate) fn pos(b: &Board) -> Option<TilePos> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use rand2048::tile_value;
    #[test]
    fn generate_tile_value() {
        for _ in 0..1000 {
            let a = tile_value();
            assert!(a == 1 || a == 2);
        }
    }
}
